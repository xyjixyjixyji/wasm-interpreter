use crate::module::module::WasmModule;

pub struct WasmInterpreter<'a> {
    module: WasmModule<'a>,
}

impl<'a> WasmInterpreter<'a> {
    pub fn from_module(module: WasmModule<'a>) -> Self {
        WasmInterpreter { module }
    }
}
