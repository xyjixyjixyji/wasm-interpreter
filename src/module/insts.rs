use anyhow::Result;
use wasmparser::{BinaryReader, BlockType, WasmFeatures};

use super::wasmops::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BrTable {
    targets: Vec<u32>,
    default_target: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemArg {
    offset: u32,
    align: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum I32Unops {
    Clz,
    Ctz,
    Popcnt,
    Extend8S,
    Extend16S,
    F64ConvertI32S,
    F64ConvertI32U,
}

#[derive(Debug, Clone, PartialEq)]
pub enum F64Unops {
    Abs,
    Neg,
    Ceil,
    Floor,
    Trunc,
    Nearest,
    Sqrt,
    I32TruncF64S,
    I32TruncF64U,
}

#[derive(Debug, Clone, PartialEq)]
pub enum I32Binops {
    Eq,
    Eqz,
    Ne,
    LtS,
    LtU,
    GtS,
    GtU,
    LeS,
    LeU,
    GeS,
    GeU,
    Add,
    Sub,
    Mul,
    DivS,
    DivU,
    RemS,
    RemU,
    And,
    Or,
    Xor,
    Shl,
    ShrS,
    ShrU,
    Rotl,
    Rotr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum F64Binops {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instructions {
    Unreachable,
    Nop,
    // control flow
    Block { ty: BlockType },
    Loop { ty: BlockType },
    If { ty: BlockType },
    Else,
    End,
    Br { rel_depth: u32 },
    BrIf { rel_depth: u32 },
    BrTable { table: BrTable },
    Return,
    Call { func_idx: u32 },
    CallIndirect { type_index: u32, table_index: u32 },
    // variable
    Drop,
    Select,
    LocalGet { local_idx: u32 },
    LocalSet { local_idx: u32 },
    LocalTee { local_idx: u32 },
    GlobalGet { global_idx: u32 },
    GlobalSet { global_idx: u32 },
    // memory
    I32Load { memarg: MemArg },
    I64Load { memarg: MemArg },
    F32Load { memarg: MemArg },
    F64Load { memarg: MemArg },
    I32Load8S { memarg: MemArg },
    I32Load8U { memarg: MemArg },
    I32Load16S { memarg: MemArg },
    I32Load16U { memarg: MemArg },
    I32Store { memarg: MemArg },
    F64Store { memarg: MemArg },
    I32Store8 { memarg: MemArg },
    I32Store16 { memarg: MemArg },
    MemorySize { mem: u32 },
    MemoryGrow { mem: u32 },
    I32Const { value: i32 },
    F64Const { value: f64 },
    // arithmetic
    I32Unop(I32Unops),
    I32BinOp(I32Binops),
    F64Unop(F64Unops),
    F64BinOp(F64Binops),
}

impl Instructions {
    pub fn from_code_bytes(code_bytes: Vec<u8>) -> Result<Vec<Instructions>> {
        let mut insts = vec![];
        let mut binary_reader = BinaryReader::new(&code_bytes, 0, WasmFeatures::all());

        while !binary_reader.eof() {
            let opcode = binary_reader.read_var_u32()?;
            match opcode {
                WASM_OP_UNREACHABLE => insts.push(Instructions::Unreachable),
                WASM_OP_NOP => insts.push(Instructions::Nop),
                WASM_OP_BLOCK => insts.push(Instructions::Block {
                    ty: Self::read_block_type(&mut binary_reader)?,
                }),
                WASM_OP_LOOP => insts.push(Instructions::Loop {
                    ty: Self::read_block_type(&mut binary_reader)?,
                }),
                WASM_OP_IF => insts.push(Instructions::If {
                    ty: Self::read_block_type(&mut binary_reader)?,
                }),
                WASM_OP_ELSE => insts.push(Instructions::Else),
                WASM_OP_END => insts.push(Instructions::End),
                WASM_OP_BR => insts.push(Instructions::Br {
                    rel_depth: binary_reader.read_var_u32()?,
                }),
                WASM_OP_BR_IF => insts.push(Instructions::BrIf {
                    rel_depth: binary_reader.read_var_u32()?,
                }),
                WASM_OP_BR_TABLE => insts.push(Instructions::BrTable {
                    table: Self::read_br_table(&mut binary_reader)?,
                }),
                WASM_OP_RETURN => insts.push(Instructions::Return),
                WASM_OP_CALL => insts.push(Instructions::Call {
                    func_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_CALL_INDIRECT => insts.push(Instructions::CallIndirect {
                    type_index: binary_reader.read_var_u32()?,
                    table_index: binary_reader.read_var_u32()?,
                }),
                WASM_OP_DROP => insts.push(Instructions::Drop),
                WASM_OP_SELECT => insts.push(Instructions::Select),
                WASM_OP_LOCAL_GET => insts.push(Instructions::LocalGet {
                    local_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_LOCAL_SET => insts.push(Instructions::LocalSet {
                    local_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_LOCAL_TEE => insts.push(Instructions::LocalTee {
                    local_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_GLOBAL_GET => insts.push(Instructions::GlobalGet {
                    global_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_GLOBAL_SET => insts.push(Instructions::GlobalSet {
                    global_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_I32_LOAD => insts.push(Instructions::I32Load {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I64_LOAD => insts.push(Instructions::I64Load {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_F32_LOAD => insts.push(Instructions::F32Load {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_F64_LOAD => insts.push(Instructions::F64Load {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_LOAD8_S => insts.push(Instructions::I32Load8S {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_LOAD8_U => insts.push(Instructions::I32Load8U {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_LOAD16_S => insts.push(Instructions::I32Load16S {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_LOAD16_U => insts.push(Instructions::I32Load16U {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_STORE => insts.push(Instructions::I32Store {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_F64_STORE => insts.push(Instructions::F64Store {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_STORE8 => insts.push(Instructions::I32Store8 {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_STORE16 => insts.push(Instructions::I32Store16 {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_MEMORY_SIZE => insts.push(Instructions::MemorySize {
                    mem: binary_reader.read_var_u32()?, // always 0
                }),
                WASM_OP_MEMORY_GROW => insts.push(Instructions::MemoryGrow {
                    mem: binary_reader.read_var_u32()?, // always 0
                }),
                WASM_OP_I32_CONST => insts.push(Instructions::I32Const {
                    value: binary_reader.read_var_i32()?,
                }),
                WASM_OP_F64_CONST => insts.push(Instructions::F64Const {
                    value: f64::from(binary_reader.read_f64()?),
                }),
                WASM_OP_I32_EQZ => insts.push(Instructions::I32BinOp(I32Binops::Eqz)),
                WASM_OP_I32_EQ => insts.push(Instructions::I32BinOp(I32Binops::Eq)),
                WASM_OP_I32_NE => insts.push(Instructions::I32BinOp(I32Binops::Ne)),
                WASM_OP_I32_LT_S => insts.push(Instructions::I32BinOp(I32Binops::LtS)),
                WASM_OP_I32_LT_U => insts.push(Instructions::I32BinOp(I32Binops::LtU)),
                WASM_OP_I32_GT_S => insts.push(Instructions::I32BinOp(I32Binops::GtS)),
                WASM_OP_I32_GT_U => insts.push(Instructions::I32BinOp(I32Binops::GtU)),
                WASM_OP_I32_LE_S => insts.push(Instructions::I32BinOp(I32Binops::LeS)),
                WASM_OP_I32_LE_U => insts.push(Instructions::I32BinOp(I32Binops::LeU)),
                WASM_OP_I32_GE_S => insts.push(Instructions::I32BinOp(I32Binops::GeS)),
                WASM_OP_I32_GE_U => insts.push(Instructions::I32BinOp(I32Binops::GeU)),
                WASM_OP_F64_EQ => insts.push(Instructions::F64BinOp(F64Binops::Eq)),
                WASM_OP_F64_NE => insts.push(Instructions::F64BinOp(F64Binops::Ne)),
                WASM_OP_F64_LT => insts.push(Instructions::F64BinOp(F64Binops::Lt)),
                WASM_OP_F64_GT => insts.push(Instructions::F64BinOp(F64Binops::Gt)),
                WASM_OP_F64_LE => insts.push(Instructions::F64BinOp(F64Binops::Le)),
                WASM_OP_F64_GE => insts.push(Instructions::F64BinOp(F64Binops::Ge)),
                WASM_OP_I32_CLZ => insts.push(Instructions::I32Unop(I32Unops::Clz)),
                WASM_OP_I32_CTZ => insts.push(Instructions::I32Unop(I32Unops::Ctz)),
                WASM_OP_I32_POPCNT => insts.push(Instructions::I32Unop(I32Unops::Popcnt)),
                WASM_OP_I32_ADD => insts.push(Instructions::I32BinOp(I32Binops::Add)),
                WASM_OP_I32_SUB => insts.push(Instructions::I32BinOp(I32Binops::Sub)),
                WASM_OP_I32_MUL => insts.push(Instructions::I32BinOp(I32Binops::Mul)),
                WASM_OP_I32_DIV_S => insts.push(Instructions::I32BinOp(I32Binops::DivS)),
                WASM_OP_I32_DIV_U => insts.push(Instructions::I32BinOp(I32Binops::DivU)),
                WASM_OP_I32_REM_S => insts.push(Instructions::I32BinOp(I32Binops::RemS)),
                WASM_OP_I32_REM_U => insts.push(Instructions::I32BinOp(I32Binops::RemU)),
                WASM_OP_I32_AND => insts.push(Instructions::I32BinOp(I32Binops::And)),
                WASM_OP_I32_OR => insts.push(Instructions::I32BinOp(I32Binops::Or)),
                WASM_OP_I32_XOR => insts.push(Instructions::I32BinOp(I32Binops::Xor)),
                WASM_OP_I32_SHL => insts.push(Instructions::I32BinOp(I32Binops::Shl)),
                WASM_OP_I32_SHR_S => insts.push(Instructions::I32BinOp(I32Binops::ShrS)),
                WASM_OP_I32_SHR_U => insts.push(Instructions::I32BinOp(I32Binops::ShrU)),
                WASM_OP_I32_ROTL => insts.push(Instructions::I32BinOp(I32Binops::Rotl)),
                WASM_OP_I32_ROTR => insts.push(Instructions::I32BinOp(I32Binops::Rotr)),
                WASM_OP_F64_ABS => insts.push(Instructions::F64Unop(F64Unops::Abs)),
                WASM_OP_F64_NEG => insts.push(Instructions::F64Unop(F64Unops::Neg)),
                WASM_OP_F64_CEIL => insts.push(Instructions::F64Unop(F64Unops::Ceil)),
                WASM_OP_F64_FLOOR => insts.push(Instructions::F64Unop(F64Unops::Floor)),
                WASM_OP_F64_TRUNC => insts.push(Instructions::F64Unop(F64Unops::Trunc)),
                WASM_OP_F64_NEAREST => insts.push(Instructions::F64Unop(F64Unops::Nearest)),
                WASM_OP_F64_SQRT => insts.push(Instructions::F64Unop(F64Unops::Sqrt)),
                WASM_OP_F64_ADD => insts.push(Instructions::F64BinOp(F64Binops::Add)),
                WASM_OP_F64_SUB => insts.push(Instructions::F64BinOp(F64Binops::Sub)),
                WASM_OP_F64_MUL => insts.push(Instructions::F64BinOp(F64Binops::Mul)),
                WASM_OP_F64_DIV => insts.push(Instructions::F64BinOp(F64Binops::Div)),
                WASM_OP_F64_MIN => insts.push(Instructions::F64BinOp(F64Binops::Min)),
                WASM_OP_F64_MAX => insts.push(Instructions::F64BinOp(F64Binops::Max)),
                WASM_OP_I32_TRUNC_F64_S => {
                    insts.push(Instructions::F64Unop(F64Unops::I32TruncF64S))
                }
                WASM_OP_I32_TRUNC_F64_U => {
                    insts.push(Instructions::F64Unop(F64Unops::I32TruncF64U))
                }
                WASM_OP_F64_CONVERT_I32_S => {
                    insts.push(Instructions::I32Unop(I32Unops::F64ConvertI32S))
                }
                WASM_OP_F64_CONVERT_I32_U => {
                    insts.push(Instructions::I32Unop(I32Unops::F64ConvertI32U))
                }
                WASM_OP_I32_EXTEND8_S => insts.push(Instructions::I32Unop(I32Unops::Extend8S)),
                WASM_OP_I32_EXTEND16_S => insts.push(Instructions::I32Unop(I32Unops::Extend16S)),
                _ => anyhow::bail!("unsupported opcode: 0x{:x}", opcode),
            }
        }

        Ok(insts)
    }

    fn read_block_type(binary_reader: &mut BinaryReader) -> Result<BlockType> {
        let mut peek_reader = binary_reader.clone();
        let b = peek_reader.read_u8()?;
        let is_neg = b & 0x80 == 0 && b & 0x40 != 0;

        if is_neg {
            // singular type
            if b == 0x40 {
                Ok(BlockType::Empty)
            } else {
                Ok(BlockType::Type(binary_reader.read()?))
            }
        } else {
            // not a singular type
            let type_index = u32::try_from(binary_reader.read_var_s33()?)?;
            Ok(BlockType::FuncType(type_index))
        }
    }

    fn read_br_table(binary_reader: &mut BinaryReader) -> Result<BrTable> {
        let count = binary_reader.read_var_u32()?;
        let mut targets = vec![];
        for _ in 0..count {
            targets.push(binary_reader.read_var_u32()?);
        }
        let default_target = binary_reader.read_var_u32()?;
        Ok(BrTable {
            targets,
            default_target,
        })
    }

    fn read_memarg(binary_reader: &mut BinaryReader) -> Result<MemArg> {
        let offset = binary_reader.read_var_u32()?;
        let align = binary_reader.read_var_u32()?;
        Ok(MemArg { offset, align })
    }
}
