use std::rc::Rc;

use anyhow::Result;
use debug_cell::RefCell;
use monoasm::*;
use monoasm_macro::monoasm;
use regalloc::{Register, REG_TEMP};

use crate::module::{value_type::WasmValue, wasm_module::WasmModule};

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
    fn compile(
        &mut self,
        module: Rc<RefCell<WasmModule>>,
        initial_mem_size_in_byte: u64,
        main_params: Vec<WasmValue>,
    ) -> Result<CodePtr>;
}

