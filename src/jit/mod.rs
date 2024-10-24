use anyhow::Result;
use monoasm::*;

use crate::module::value_type::WasmValue;

pub use compiler::X86JitCompiler;
pub use mem::JitLinearMemory;
pub use setup::trap::register_trap_handler;

pub type ReturnFunc = extern "C" fn() -> u64;

mod compiler;
mod insts;
mod mem;
mod regalloc;
mod setup;
mod utils;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ValueType {
    I32,
    F64,
}

pub trait WasmJitCompiler {
    fn compile(&mut self, main_params: Vec<WasmValue>) -> Result<CodePtr>;
}
