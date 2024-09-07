use debug_cell::RefCell;
use std::rc::Rc;

use crate::{
    module::{value_type::WasmValue, wasm_module::WasmModule},
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
        let mem = LinearMemory(if let Some(mem) = module.get_memory() {
            vec![0; mem.initial as usize * WASM_DEFAULT_PAGE_SIZE_BYTE]
        } else {
            vec![]
        });

        WasmInterpreter {
            module: Rc::new(RefCell::new(module)),
            mem: Rc::new(RefCell::new(mem)),
        }
    }

    pub fn mem_size(&self) -> usize {
        self.mem.borrow().size()
    }

    pub fn grow_mem(&mut self, additional_pages: u32) {
        self.mem.borrow_mut().grow(additional_pages);
    }
}
