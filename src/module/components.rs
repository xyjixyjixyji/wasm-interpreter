use super::insts::Instructions;
use wasmparser::{FuncType, Import, ValType};

#[derive(Default)]
pub struct ImportSet<'a> {
    pub imports: Vec<Import<'a>>,
    pub num_funcs: u32,
    pub num_tables: u32,
    pub num_mems: u32,
    pub num_globals: u32,
}

impl<'a> ImportSet<'a> {
    pub fn get_num_imports(&self) -> usize {
        self.imports.len()
    }
}

#[derive(Debug, Clone)]
pub struct FuncDecl {
    sig: FuncType,
    pure_locals: Vec<(u32, ValType)>,
    insts: Vec<Instructions>,
}

impl FuncDecl {
    pub fn new(sig: FuncType) -> Self {
        Self {
            sig,
            pure_locals: vec![],
            insts: vec![],
        }
    }

    pub fn get_sig(&self) -> &FuncType {
        &self.sig
    }

    pub fn get_pure_locals(&self) -> &[(u32, ValType)] {
        &self.pure_locals
    }

    pub fn add_pure_local(&mut self, local: (u32, ValType)) {
        self.pure_locals.push(local);
    }
}
