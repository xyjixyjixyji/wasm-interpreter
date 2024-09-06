use anyhow::Result;
use wasmparser::{BinaryReader, BlockType, MemArg, WasmFeatures};

use super::wasmops::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BrTable {
    targets: Vec<u32>,
    default_target: u32,
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
            let opcode = binary_reader.read_u32()?;
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
                _ => anyhow::bail!("unsupported opcode: {}", opcode),
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
}
