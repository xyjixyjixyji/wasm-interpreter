use std::rc::Rc;

use anyhow::Result;
use debug_cell::RefCell;
use monoasm::CodePtr;

use crate::module::wasm_module::WasmModule;

pub use compiler::X86JitCompiler;
pub use mem::JitLinearMemory;
pub use trap::register_trap_handler;

pub type I32ReturnFunc = extern "C" fn() -> i32;
pub type F64ReturnFunc = extern "C" fn() -> f64;

mod compiler;
mod mem;
mod regalloc;
mod trap;

pub trait WasmJitCompiler {
    fn compile(
        &mut self,
        module: Rc<RefCell<WasmModule>>,
        initial_mem_size_in_byte: u64,
    ) -> Result<CodePtr>;
}
