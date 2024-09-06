use anyhow::Result;

mod interpreter;
pub use interpreter::WasmInterpreter;
mod exec;

pub trait WasmVm {
    /// Run the interpreter,the final result will be returned as a string.
    fn run(&self) -> Result<String>;
}
