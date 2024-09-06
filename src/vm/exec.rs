use super::{WasmInterpreter, WasmVm};

impl<'a> WasmVm for WasmInterpreter<'a> {
    fn run(&self) -> anyhow::Result<String> {
        Ok("".to_string())
    }
}
