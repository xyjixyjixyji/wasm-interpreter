use anyhow::Result;

use crate::module::value_type::WasmValue;

mod interpreter;
pub use interpreter::WasmInterpreter;

mod exec;
mod func_exec;

const WASM_DEFAULT_PAGE_SIZE_BYTE: usize = 65536;

pub trait WasmVm {
    /// Run the interpreter,the final result will be returned as a string.
    fn run(&self) -> Result<String>;
}

pub trait WasmFunctionExecutor {
    fn execute(&mut self) -> Result<WasmValue>;
}
