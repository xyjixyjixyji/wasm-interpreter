use wasmparser::{FuncType, Import, Operator, ValType};

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

#[derive(Clone)]
pub struct FuncDecl<'a> {
    sig: FuncType,
    pure_locals: Vec<(u32, ValType)>,
    operators: Vec<Operator<'a>>,
}

impl<'a> FuncDecl<'a> {
    pub fn new(sig: FuncType) -> Self {
        Self {
            sig,
            pure_locals: vec![],
            operators: vec![],
        }
    }

    pub fn get_sig(&self) -> &FuncType {
        &self.sig
    }

    pub fn get_pure_locals(&self) -> &[(u32, ValType)] {
        &self.pure_locals
    }

    pub fn get_operators(&self) -> &[Operator] {
        &self.operators
    }

    pub fn add_operator(&mut self, op: Operator<'a>) {
        self.operators.push(op);
    }

    pub fn add_pure_local(&mut self, local: (u32, ValType)) {
        self.pure_locals.push(local);
    }
}
