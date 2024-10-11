use anyhow::Result;
use wasmparser::{BinaryReader, BlockType, WasmFeatures};

use super::wasmops::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BrTable {
    pub targets: Vec<u32>,
    pub default_target: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum I32Unop {
    Eqz,
    Clz,
    Ctz,
    Popcnt,
    Extend8S,
    Extend16S,
    F64ConvertI32S,
    F64ConvertI32U,
}

#[derive(Debug, Clone, PartialEq)]
pub enum I32Binop {
    Eq,
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
pub enum F64Unop {
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
pub enum F64Binop {
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
pub enum Instruction {
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
    I32Unop(I32Unop),
    I32Binop(I32Binop),
    F64Unop(F64Unop),
    F64Binop(F64Binop),
}

impl Instruction {
    pub fn from_code_bytes(code_bytes: Vec<u8>) -> Result<Vec<Instruction>> {
        let mut insts = vec![];
        let mut binary_reader = BinaryReader::new(&code_bytes, 0, WasmFeatures::all());

        while !binary_reader.eof() {
            // legal opcodes are u8 operators, so we can just read u8
            let opcode = binary_reader.read_u8()? as u32;
            match opcode {
                WASM_OP_UNREACHABLE => insts.push(Instruction::Unreachable),
                WASM_OP_NOP => insts.push(Instruction::Nop),
                WASM_OP_BLOCK => insts.push(Instruction::Block {
                    ty: Self::read_block_type(&mut binary_reader)?,
                }),
                WASM_OP_LOOP => insts.push(Instruction::Loop {
                    ty: Self::read_block_type(&mut binary_reader)?,
                }),
                WASM_OP_IF => insts.push(Instruction::If {
                    ty: Self::read_block_type(&mut binary_reader)?,
                }),
                WASM_OP_ELSE => insts.push(Instruction::Else),
                WASM_OP_END => insts.push(Instruction::End),
                WASM_OP_BR => insts.push(Instruction::Br {
                    rel_depth: binary_reader.read_var_u32()?,
                }),
                WASM_OP_BR_IF => insts.push(Instruction::BrIf {
                    rel_depth: binary_reader.read_var_u32()?,
                }),
                WASM_OP_BR_TABLE => insts.push(Instruction::BrTable {
                    table: Self::read_br_table(&mut binary_reader)?,
                }),
                WASM_OP_RETURN => insts.push(Instruction::Return),
                WASM_OP_CALL => insts.push(Instruction::Call {
                    func_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_CALL_INDIRECT => insts.push(Instruction::CallIndirect {
                    type_index: binary_reader.read_var_u32()?,
                    table_index: binary_reader.read_var_u32()?,
                }),
                WASM_OP_DROP => insts.push(Instruction::Drop),
                WASM_OP_SELECT => insts.push(Instruction::Select),
                WASM_OP_LOCAL_GET => insts.push(Instruction::LocalGet {
                    local_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_LOCAL_SET => insts.push(Instruction::LocalSet {
                    local_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_LOCAL_TEE => insts.push(Instruction::LocalTee {
                    local_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_GLOBAL_GET => insts.push(Instruction::GlobalGet {
                    global_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_GLOBAL_SET => insts.push(Instruction::GlobalSet {
                    global_idx: binary_reader.read_var_u32()?,
                }),
                WASM_OP_I32_LOAD => insts.push(Instruction::I32Load {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_F64_LOAD => insts.push(Instruction::F64Load {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_LOAD8_S => insts.push(Instruction::I32Load8S {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_LOAD8_U => insts.push(Instruction::I32Load8U {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_LOAD16_S => insts.push(Instruction::I32Load16S {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_LOAD16_U => insts.push(Instruction::I32Load16U {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_STORE => insts.push(Instruction::I32Store {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_F64_STORE => insts.push(Instruction::F64Store {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_STORE8 => insts.push(Instruction::I32Store8 {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_I32_STORE16 => insts.push(Instruction::I32Store16 {
                    memarg: Self::read_memarg(&mut binary_reader)?,
                }),
                WASM_OP_MEMORY_SIZE => insts.push(Instruction::MemorySize {
                    mem: binary_reader.read_var_u32()?, // always 0
                }),
                WASM_OP_MEMORY_GROW => insts.push(Instruction::MemoryGrow {
                    mem: binary_reader.read_var_u32()?, // always 0
                }),
                WASM_OP_I32_CONST => insts.push(Instruction::I32Const {
                    value: binary_reader.read_var_i32()?,
                }),
                WASM_OP_F64_CONST => insts.push(Instruction::F64Const {
                    value: f64::from(binary_reader.read_f64()?),
                }),
                WASM_OP_I32_EQZ => insts.push(Instruction::I32Unop(I32Unop::Eqz)),
                WASM_OP_I32_EQ => insts.push(Instruction::I32Binop(I32Binop::Eq)),
                WASM_OP_I32_NE => insts.push(Instruction::I32Binop(I32Binop::Ne)),
                WASM_OP_I32_LT_S => insts.push(Instruction::I32Binop(I32Binop::LtS)),
                WASM_OP_I32_LT_U => insts.push(Instruction::I32Binop(I32Binop::LtU)),
                WASM_OP_I32_GT_S => insts.push(Instruction::I32Binop(I32Binop::GtS)),
                WASM_OP_I32_GT_U => insts.push(Instruction::I32Binop(I32Binop::GtU)),
                WASM_OP_I32_LE_S => insts.push(Instruction::I32Binop(I32Binop::LeS)),
                WASM_OP_I32_LE_U => insts.push(Instruction::I32Binop(I32Binop::LeU)),
                WASM_OP_I32_GE_S => insts.push(Instruction::I32Binop(I32Binop::GeS)),
                WASM_OP_I32_GE_U => insts.push(Instruction::I32Binop(I32Binop::GeU)),
                WASM_OP_F64_EQ => insts.push(Instruction::F64Binop(F64Binop::Eq)),
                WASM_OP_F64_NE => insts.push(Instruction::F64Binop(F64Binop::Ne)),
                WASM_OP_F64_LT => insts.push(Instruction::F64Binop(F64Binop::Lt)),
                WASM_OP_F64_GT => insts.push(Instruction::F64Binop(F64Binop::Gt)),
                WASM_OP_F64_LE => insts.push(Instruction::F64Binop(F64Binop::Le)),
                WASM_OP_F64_GE => insts.push(Instruction::F64Binop(F64Binop::Ge)),
                WASM_OP_I32_CLZ => insts.push(Instruction::I32Unop(I32Unop::Clz)),
                WASM_OP_I32_CTZ => insts.push(Instruction::I32Unop(I32Unop::Ctz)),
                WASM_OP_I32_POPCNT => insts.push(Instruction::I32Unop(I32Unop::Popcnt)),
                WASM_OP_I32_ADD => insts.push(Instruction::I32Binop(I32Binop::Add)),
                WASM_OP_I32_SUB => insts.push(Instruction::I32Binop(I32Binop::Sub)),
                WASM_OP_I32_MUL => insts.push(Instruction::I32Binop(I32Binop::Mul)),
                WASM_OP_I32_DIV_S => insts.push(Instruction::I32Binop(I32Binop::DivS)),
                WASM_OP_I32_DIV_U => insts.push(Instruction::I32Binop(I32Binop::DivU)),
                WASM_OP_I32_REM_S => insts.push(Instruction::I32Binop(I32Binop::RemS)),
                WASM_OP_I32_REM_U => insts.push(Instruction::I32Binop(I32Binop::RemU)),
                WASM_OP_I32_AND => insts.push(Instruction::I32Binop(I32Binop::And)),
                WASM_OP_I32_OR => insts.push(Instruction::I32Binop(I32Binop::Or)),
                WASM_OP_I32_XOR => insts.push(Instruction::I32Binop(I32Binop::Xor)),
                WASM_OP_I32_SHL => insts.push(Instruction::I32Binop(I32Binop::Shl)),
                WASM_OP_I32_SHR_S => insts.push(Instruction::I32Binop(I32Binop::ShrS)),
                WASM_OP_I32_SHR_U => insts.push(Instruction::I32Binop(I32Binop::ShrU)),
                WASM_OP_I32_ROTL => insts.push(Instruction::I32Binop(I32Binop::Rotl)),
                WASM_OP_I32_ROTR => insts.push(Instruction::I32Binop(I32Binop::Rotr)),
                WASM_OP_F64_ABS => insts.push(Instruction::F64Unop(F64Unop::Abs)),
                WASM_OP_F64_NEG => insts.push(Instruction::F64Unop(F64Unop::Neg)),
                WASM_OP_F64_CEIL => insts.push(Instruction::F64Unop(F64Unop::Ceil)),
                WASM_OP_F64_FLOOR => insts.push(Instruction::F64Unop(F64Unop::Floor)),
                WASM_OP_F64_TRUNC => insts.push(Instruction::F64Unop(F64Unop::Trunc)),
                WASM_OP_F64_NEAREST => insts.push(Instruction::F64Unop(F64Unop::Nearest)),
                WASM_OP_F64_SQRT => insts.push(Instruction::F64Unop(F64Unop::Sqrt)),
                WASM_OP_F64_ADD => insts.push(Instruction::F64Binop(F64Binop::Add)),
                WASM_OP_F64_SUB => insts.push(Instruction::F64Binop(F64Binop::Sub)),
                WASM_OP_F64_MUL => insts.push(Instruction::F64Binop(F64Binop::Mul)),
                WASM_OP_F64_DIV => insts.push(Instruction::F64Binop(F64Binop::Div)),
                WASM_OP_F64_MIN => insts.push(Instruction::F64Binop(F64Binop::Min)),
                WASM_OP_F64_MAX => insts.push(Instruction::F64Binop(F64Binop::Max)),
                WASM_OP_I32_TRUNC_F64_S => insts.push(Instruction::F64Unop(F64Unop::I32TruncF64S)),
                WASM_OP_I32_TRUNC_F64_U => insts.push(Instruction::F64Unop(F64Unop::I32TruncF64U)),
                WASM_OP_F64_CONVERT_I32_S => {
                    insts.push(Instruction::I32Unop(I32Unop::F64ConvertI32S))
                }
                WASM_OP_F64_CONVERT_I32_U => {
                    insts.push(Instruction::I32Unop(I32Unop::F64ConvertI32U))
                }
                WASM_OP_I32_EXTEND8_S => insts.push(Instruction::I32Unop(I32Unop::Extend8S)),
                WASM_OP_I32_EXTEND16_S => insts.push(Instruction::I32Unop(I32Unop::Extend16S)),
                _ => anyhow::bail!("unsupported opcode: 0x{:x}", opcode),
            }
        }

        Ok(insts)
    }

    pub fn is_control_block_start(inst: &Instruction) -> bool {
        matches!(
            inst,
            Instruction::Block { .. } | Instruction::Loop { .. } | Instruction::If { .. }
        )
    }

    pub fn is_control_block_end(inst: &Instruction) -> bool {
        matches!(inst, Instruction::End)
    }

    fn read_block_type(binary_reader: &mut BinaryReader) -> Result<BlockType> {
        let mut peek_reader = binary_reader.clone();
        let b = peek_reader.read_u8()?;
        let is_neg = b & 0x80 == 0 && b & 0x40 != 0;

        if is_neg {
            // singular type
            if b == 0x40 {
                binary_reader.read_u8()?;
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
        let align = binary_reader.read_var_u32()?;
        let offset = binary_reader.read_var_u32()?;
        Ok(MemArg { offset, align })
    }
}
