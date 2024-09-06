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
pub enum Instructions {
    Unreachable,
    Nop,
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
    Drop,
    Select,
    LocalGet { local_idx: u32 },
    LocalSet { local_idx: u32 },
    LocalTee { local_idx: u32 },
    GlobalGet { global_idx: u32 },
    GlobalSet { global_idx: u32 },
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
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    I32TruncF64S,
    I32TruncF64U,
    F64ConvertI32S,
    F64ConvertI32U,
    I32Extend8S,
    I32Extend16S,
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
                WASM_OP_I32_EQZ => insts.push(Instructions::I32Eqz),
                WASM_OP_I32_EQ => insts.push(Instructions::I32Eq),
                WASM_OP_I32_NE => insts.push(Instructions::I32Ne),
                WASM_OP_I32_LT_S => insts.push(Instructions::I32LtS),
                WASM_OP_I32_LT_U => insts.push(Instructions::I32LtU),
                WASM_OP_I32_GT_S => insts.push(Instructions::I32GtS),
                WASM_OP_I32_GT_U => insts.push(Instructions::I32GtU),
                WASM_OP_I32_LE_S => insts.push(Instructions::I32LeS),
                WASM_OP_I32_LE_U => insts.push(Instructions::I32LeU),
                WASM_OP_I32_GE_S => insts.push(Instructions::I32GeS),
                WASM_OP_I32_GE_U => insts.push(Instructions::I32GeU),
                WASM_OP_F64_EQ => insts.push(Instructions::F64Eq),
                WASM_OP_F64_NE => insts.push(Instructions::F64Ne),
                WASM_OP_F64_LT => insts.push(Instructions::F64Lt),
                WASM_OP_F64_GT => insts.push(Instructions::F64Gt),
                WASM_OP_F64_LE => insts.push(Instructions::F64Le),
                WASM_OP_F64_GE => insts.push(Instructions::F64Ge),
                WASM_OP_I32_CLZ => insts.push(Instructions::I32Clz),
                WASM_OP_I32_CTZ => insts.push(Instructions::I32Ctz),
                WASM_OP_I32_POPCNT => insts.push(Instructions::I32Popcnt),
                WASM_OP_I32_ADD => insts.push(Instructions::I32Add),
                WASM_OP_I32_SUB => insts.push(Instructions::I32Sub),
                WASM_OP_I32_MUL => insts.push(Instructions::I32Mul),
                WASM_OP_I32_DIV_S => insts.push(Instructions::I32DivS),
                WASM_OP_I32_DIV_U => insts.push(Instructions::I32DivU),
                WASM_OP_I32_REM_S => insts.push(Instructions::I32RemS),
                WASM_OP_I32_REM_U => insts.push(Instructions::I32RemU),
                WASM_OP_I32_AND => insts.push(Instructions::I32And),
                WASM_OP_I32_OR => insts.push(Instructions::I32Or),
                WASM_OP_I32_XOR => insts.push(Instructions::I32Xor),
                WASM_OP_I32_SHL => insts.push(Instructions::I32Shl),
                WASM_OP_I32_SHR_S => insts.push(Instructions::I32ShrS),
                WASM_OP_I32_SHR_U => insts.push(Instructions::I32ShrU),
                WASM_OP_I32_ROTL => insts.push(Instructions::I32Rotl),
                WASM_OP_I32_ROTR => insts.push(Instructions::I32Rotr),
                WASM_OP_F64_ABS => insts.push(Instructions::F64Abs),
                WASM_OP_F64_NEG => insts.push(Instructions::F64Neg),
                WASM_OP_F64_CEIL => insts.push(Instructions::F64Ceil),
                WASM_OP_F64_FLOOR => insts.push(Instructions::F64Floor),
                WASM_OP_F64_TRUNC => insts.push(Instructions::F64Trunc),
                WASM_OP_F64_NEAREST => insts.push(Instructions::F64Nearest),
                WASM_OP_F64_SQRT => insts.push(Instructions::F64Sqrt),
                WASM_OP_F64_ADD => insts.push(Instructions::F64Add),
                WASM_OP_F64_SUB => insts.push(Instructions::F64Sub),
                WASM_OP_F64_MUL => insts.push(Instructions::F64Mul),
                WASM_OP_F64_DIV => insts.push(Instructions::F64Div),
                WASM_OP_F64_MIN => insts.push(Instructions::F64Min),
                WASM_OP_F64_MAX => insts.push(Instructions::F64Max),
                WASM_OP_I32_TRUNC_F64_S => insts.push(Instructions::I32TruncF64S),
                WASM_OP_I32_TRUNC_F64_U => insts.push(Instructions::I32TruncF64U),
                WASM_OP_F64_CONVERT_I32_S => insts.push(Instructions::F64ConvertI32S),
                WASM_OP_F64_CONVERT_I32_U => insts.push(Instructions::F64ConvertI32U),
                WASM_OP_I32_EXTEND8_S => insts.push(Instructions::I32Extend8S),
                WASM_OP_I32_EXTEND16_S => insts.push(Instructions::I32Extend16S),
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
