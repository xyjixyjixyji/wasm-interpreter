use anyhow::Result;
use monoasm::CodePtr;

use crate::module::components::FuncDecl;

pub use compiler::X86JitCompiler;

mod compiler;
mod regalloc;

pub type I32ReturnFunc = fn() -> i32;

pub trait WasmJitCompiler {
    fn compile(&mut self, fdecl: &FuncDecl) -> Result<CodePtr>;
}
