use super::{insts::Instruction, parse::FuncBody};
use wasmparser::{FuncType, GlobalType, Import, ValType};

#[derive(Default, Debug)]
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
    insts: Vec<Instruction>,
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

    pub fn get_insts(&self) -> &Vec<Instruction> {
        &self.insts
    }

    pub fn get_inst(&self, idx: usize) -> &Instruction {
        &self.insts[idx]
    }

    pub fn add_func_body(&mut self, func_body: FuncBody) {
        self.pure_locals = func_body.locals;
        self.insts = func_body.insts;
    }
}

#[derive(Debug, Clone)]
pub struct GlobalDecl {
    ty: GlobalType,
    init_expr: Vec<u8>,
}

impl GlobalDecl {
    pub fn new(ty: GlobalType, init_expr: Vec<u8>) -> Self {
        Self { ty, init_expr }
    }

    pub fn get_ty(&self) -> &GlobalType {
        &self.ty
    }

    pub fn get_init_expr(&self) -> &Vec<u8> {
        &self.init_expr
    }

    pub fn set_init_expr(&mut self, init_expr: Vec<u8>) {
        self.init_expr = init_expr;
    }
}
