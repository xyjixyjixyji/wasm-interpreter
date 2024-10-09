use super::WasmJitCompiler;
use crate::module::components::FuncDecl;

use anyhow::Result;

pub struct X86JitCompiler {}

impl X86JitCompiler {
    pub fn new() -> Self {
        Self {}
    }
}

impl WasmJitCompiler for X86JitCompiler {
    fn compile(&self, fdecl: FuncDecl) -> Result<Vec<String>> {
        todo!()
    }
}
