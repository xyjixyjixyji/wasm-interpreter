use anyhow::{anyhow, Result};
use debug_cell::RefCell;

use std::rc::Rc;

use crate::{
    module::{value_type::WasmValue, wasm_module::WasmModule, wasmops::WASM_OP_I32_CONST},
    vm::WASM_DEFAULT_PAGE_SIZE_BYTE,
};

use super::{func_exec::WasmFunctionExecutorImpl, WasmFunctionExecutor, WasmVm};

pub(crate) struct LinearMemory(pub(crate) Vec<u8>);

impl LinearMemory {
    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn grow(&mut self, additional_pages: u32) {
        let new_size = self.0.len() + (additional_pages as usize * WASM_DEFAULT_PAGE_SIZE_BYTE);
        self.0.resize(new_size, 0);
    }
}

pub struct WasmInterpreter<'a> {
    module: Rc<RefCell<WasmModule<'a>>>,
    mem: Rc<RefCell<LinearMemory>>,
}

impl<'a> WasmVm for WasmInterpreter<'a> {
    fn run(&self, main_params: Vec<WasmValue>) -> anyhow::Result<String> {
        // find main from export to run
        let main_func = {
            let module_ref = self.module.borrow();
            module_ref
                .get_exports()
                .iter()
                .find(|export| export.name == "main")
                .and_then(|export| module_ref.get_func(export.index))
                .ok_or_else(|| anyhow::anyhow!("main function not found"))?
                .clone()
        };

        let mut executor = WasmFunctionExecutorImpl::new(
            main_func,
            Rc::clone(&self.module),
            Rc::clone(&self.mem),
            Some(main_params),
        );

        let result = executor.execute()?;
        Ok(result.to_string())
    }
}

impl<'a> WasmInterpreter<'a> {
    pub fn from_module(module: WasmModule<'a>) -> Self {
        let mut mem = LinearMemory(if let Some(mem) = module.get_memory() {
            vec![0; mem.initial as usize * WASM_DEFAULT_PAGE_SIZE_BYTE]
        } else {
            vec![]
        });

        Self::setup_data_section(&module, &mut mem).expect("failed to setup data section");

        WasmInterpreter {
            module: Rc::new(RefCell::new(module)),
            mem: Rc::new(RefCell::new(mem)),
        }
    }
}

impl<'a> WasmInterpreter<'a> {
    /// setup data section with the given data section in the module
    /// e.g. (data (i32.const 10) "foo") will be loaded to linear memory at address 10
    fn setup_data_section(module: &WasmModule<'a>, mem: &mut LinearMemory) -> Result<()> {
        let datas = module.get_datas();
        for data in datas {
            match &data.kind {
                wasmparser::DataKind::Passive => panic!("passive data segment not implemented"),
                wasmparser::DataKind::Active {
                    memory_index,
                    offset_expr,
                } => {
                    if *memory_index != 0 {
                        return Err(anyhow!("memory.init: invalid memory index"));
                    }

                    // read offset_index
                    let mut reader = offset_expr.get_binary_reader();
                    let op = reader.read_u8()?; // skip WASM_OP_I32_CONST
                    if op as u32 != WASM_OP_I32_CONST {
                        panic!("data segment offset: invalid opcode, should be i32.const");
                    }

                    let offset = reader.read_var_i32()?;
                    let byte_slice = data.data;

                    let offset = usize::try_from(offset)?;
                    for (i, b) in byte_slice.iter().enumerate() {
                        mem.0[offset + i] = *b;
                    }
                }
            }
        }

        Ok(())
    }
}
