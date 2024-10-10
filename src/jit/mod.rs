use std::rc::Rc;

use anyhow::Result;
use debug_cell::RefCell;
use monoasm::CodePtr;

use crate::module::{components::FuncDecl, wasm_module::WasmModule};

pub use compiler::X86JitCompiler;

mod compiler;
mod regalloc;

pub type I32ReturnFunc = extern "C" fn() -> i32;
pub type F64ReturnFunc = extern "C" fn() -> f64;

pub trait WasmJitCompiler {
    fn compile(&mut self, module: Rc<RefCell<WasmModule>>) -> Result<CodePtr>;
}
