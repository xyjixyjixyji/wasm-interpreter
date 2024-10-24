use anyhow::Result;
use wasmparser::{BinaryReader, ValType, WasmFeatures};

use crate::{
    jit::{ValueType, X86JitCompiler},
    module::wasmops::{WASM_OP_F64_CONST, WASM_OP_I32_CONST},
};

impl X86JitCompiler<'_> {
    pub(crate) fn setup_globals(&mut self) -> Result<()> {
        let module = self.module.borrow();
        let globals = module.get_globals();

        for (i, global) in globals.iter().enumerate() {
            match global.get_ty().content_type {
                ValType::I32 => {
                    self.global_types[i] = ValueType::I32;
                    let init_expr = global.get_init_expr();
                    let mut reader = BinaryReader::new(init_expr, 0, WasmFeatures::all());
                    let op = reader.read_var_u32()?;
                    if op != WASM_OP_I32_CONST {
                        panic!("global.get: invalid init expr, should start with i32.const");
                    }
                    self.globals[i] = reader.read_var_i32()? as u64;
                }
                ValType::F64 => {
                    self.global_types[i] = ValueType::F64;
                    let init_expr = global.get_init_expr();
                    let mut reader = BinaryReader::new(init_expr, 0, WasmFeatures::all());
                    let op = reader.read_var_u32()?;
                    if op != WASM_OP_F64_CONST {
                        panic!("global.get: invalid init expr, should start with f64.const");
                    }
                    self.globals[i] = f64::from(reader.read_f64()?).to_bits();
                }
                _ => panic!("unsupported global type"),
            }
        }

        Ok(())
    }
}
