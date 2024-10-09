use anyhow::Result;

use crate::module::components::FuncDecl;

mod compiler;
mod regalloc;

pub trait WasmJitCompiler {
    fn compile(&self, fdecl: FuncDecl) -> Result<Vec<String>>;
}
