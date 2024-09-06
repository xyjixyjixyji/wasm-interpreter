use crate::{module::wasm_module::WasmModule, vm::WASM_DEFAULT_PAGE_SIZE_BYTE};

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
    module: WasmModule<'a>,
    mem: LinearMemory,
}

impl<'a> WasmInterpreter<'a> {
    pub fn from_module(module: WasmModule<'a>) -> Self {
        let mem = LinearMemory(if let Some(mem) = module.get_memory() {
            vec![0; mem.initial as usize * WASM_DEFAULT_PAGE_SIZE_BYTE]
        } else {
            vec![]
        });

        WasmInterpreter { module, mem }
    }

    pub fn mem_size(&self) -> usize {
        self.mem.size()
    }

    pub fn grow_mem(&mut self, additional_pages: u32) {
        self.mem.grow(additional_pages);
    }
}
