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

pub struct FuncDecl {
    sig: FuncType,
    pure_locals: Vec<ValType>,
    bytecode: Vec<u8>,
}

impl FuncDecl {
    pub fn new(sig: FuncType) -> Self {
        Self {
            sig,
            pure_locals: vec![],
            bytecode: vec![],
        }
    }

    pub fn get_sig(&self) -> &FuncType {
        &self.sig
    }

    pub fn get_pure_locals(&self) -> &[ValType] {
        &self.pure_locals
    }

    pub fn get_bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    pub fn set_bytecode(&mut self, bytecode: Vec<u8>) {
        self.bytecode = bytecode;
    }

    pub fn add_pure_local(&mut self, local: ValType) {
        self.pure_locals.push(local);
    }
}
