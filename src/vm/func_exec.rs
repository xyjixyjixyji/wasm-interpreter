use anyhow::{anyhow, Result};

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use super::{interpreter::LinearMemory, WasmFunctionExecutor};
use crate::module::{
    components::FuncDecl,
    insts::{I32Binop, Instructions},
    value_type::WasmValue,
    wasm_module::WasmModule,
};

struct Pc(usize);

pub(super) struct ControlFlowTable {
    /// jump table, the key is the branch instruction pc, the value is the target pc.
    jump_table: HashMap<Pc, Pc>,
}

pub(crate) struct WasmFunctionExecutorImpl<'a> {
    /// The function to execute.
    func: FuncDecl,
    /// The program counter. Point into function's instructions.
    pc: Pc,
    /// The operand stack.
    operand_stack: VecDeque<WasmValue>,
    /// local variables
    locals: Vec<WasmValue>,
    /// The control flow table
    control_flow_table: ControlFlowTable,
    /// The reference to the linear memory for the Wasm VM instance.
    mem: Rc<RefCell<LinearMemory>>,
    /// The reference to the Wasm module for the Wasm VM instance.
    module: Rc<RefCell<WasmModule<'a>>>,
}

impl<'a> WasmFunctionExecutor for WasmFunctionExecutorImpl<'a> {
    fn execute(&mut self) -> Result<WasmValue> {
        let mut done_exec = false;
        while !done_exec && self.pc.0 < self.func.get_insts().len() {
            let inst = self.func.get_inst(self.pc.0).clone();
            match inst {
                Instructions::Return => {
                    done_exec = true;
                }
                Instructions::Unreachable => {
                    Err(anyhow!("unreachable instruction"))?;
                }
                Instructions::Nop => {
                    self.inc_pc();
                }
                Instructions::Block { ty } => todo!(),
                Instructions::Loop { ty } => todo!(),
                Instructions::If { ty } => todo!(),
                Instructions::Else => todo!(),
                Instructions::End => {
                    self.inc_pc();
                }
                Instructions::Br { rel_depth } => todo!(),
                Instructions::BrIf { rel_depth } => todo!(),
                Instructions::BrTable { table } => todo!(),
                Instructions::Call { func_idx } => todo!(),
                Instructions::CallIndirect {
                    type_index,
                    table_index,
                } => todo!(),
                Instructions::Drop => {
                    self.pop_operand_stack();
                    self.inc_pc();
                }
                Instructions::Select => todo!(),
                Instructions::LocalGet { local_idx } => {
                    let local = self.locals[local_idx as usize];
                    self.push_operand_stack(local);
                    self.inc_pc();
                }
                Instructions::LocalSet { local_idx } => {
                    let value = self.pop_operand_stack();
                    self.locals[local_idx as usize] = value;
                    self.inc_pc();
                }
                Instructions::LocalTee { local_idx } => {
                    let value = self.pop_operand_stack();
                    self.locals[local_idx as usize] = value;
                    self.push_operand_stack(value);
                    self.inc_pc();
                }
                Instructions::GlobalGet { global_idx } => todo!(),
                Instructions::GlobalSet { global_idx } => todo!(),
                Instructions::I32Load { memarg } => todo!(),
                Instructions::I64Load { memarg } => todo!(),
                Instructions::F32Load { memarg } => todo!(),
                Instructions::F64Load { memarg } => todo!(),
                Instructions::I32Load8S { memarg } => todo!(),
                Instructions::I32Load8U { memarg } => todo!(),
                Instructions::I32Load16S { memarg } => todo!(),
                Instructions::I32Load16U { memarg } => todo!(),
                Instructions::I32Store { memarg } => todo!(),
                Instructions::F64Store { memarg } => todo!(),
                Instructions::I32Store8 { memarg } => todo!(),
                Instructions::I32Store16 { memarg } => todo!(),
                Instructions::MemorySize { mem } => {
                    self.run_memory_size(mem)?;
                    self.inc_pc();
                }
                Instructions::MemoryGrow { mem } => {
                    self.run_memory_grow(mem)?;
                    self.inc_pc();
                }
                Instructions::I32Const { value } => {
                    self.push_operand_stack(WasmValue::I32(value));
                    self.inc_pc();
                }
                Instructions::F64Const { value } => {
                    self.push_operand_stack(WasmValue::F64(value));
                    self.inc_pc();
                }
                Instructions::I32Unop(_) => todo!(),
                Instructions::I32Binp(i32_binop) => {
                    self.run_i32_binop(&i32_binop)?;
                    self.inc_pc();
                }
                Instructions::F64Unop(_) => todo!(),
                Instructions::F64Binop(_) => todo!(),
            }
        }

        Ok(self.pop_operand_stack())
    }
}

impl<'a> WasmFunctionExecutorImpl<'a> {
    pub fn new(
        func: FuncDecl,
        module: Rc<RefCell<WasmModule<'a>>>,
        mem: Rc<RefCell<LinearMemory>>,
        main_locals: Option<Vec<WasmValue>>,
    ) -> Self {
        let control_flow_table = Self::analyze_control_flow_table(&func, Rc::clone(&module));
        let mut locals = if let Some(locals) = main_locals {
            locals
        } else {
            vec![]
        };

        locals.extend(
            func.get_pure_locals()
                .iter()
                .map(|(_, ty)| WasmValue::default_value(ty)),
        );

        Self {
            func,
            pc: Pc(0),
            mem,
            module,
            locals,
            control_flow_table,
            operand_stack: VecDeque::new(),
        }
    }

    pub fn inc_pc(&mut self) {
        self.pc.0 += 1;
    }

    pub fn push_operand_stack(&mut self, value: WasmValue) {
        self.operand_stack.push_front(value);
    }

    pub fn pop_operand_stack(&mut self) -> WasmValue {
        self.operand_stack
            .pop_front()
            .expect("operand stack underflow")
    }

    pub fn mem_size(&self) -> usize {
        self.mem.borrow().size()
    }

    pub fn grow_mem(&mut self, additional_pages: u32) {
        self.mem.borrow_mut().grow(additional_pages);
    }

    pub fn call_func(&self, func: FuncDecl) -> WasmValue {
        let mut executor = WasmFunctionExecutorImpl::new(
            func,
            Rc::clone(&self.module),
            Rc::clone(&self.mem),
            None,
        );

        executor.execute().unwrap()
    }

    // TODO
    fn analyze_control_flow_table(
        func: &FuncDecl,
        module: Rc<RefCell<WasmModule<'a>>>,
    ) -> ControlFlowTable {
        let jump_table = HashMap::new();

        ControlFlowTable { jump_table }
    }
}

/// Instruction execution
impl<'a> WasmFunctionExecutorImpl<'a> {
    fn run_memory_size(&mut self, mem: u32) -> Result<()> {
        if mem != 0 {
            return Err(anyhow!("memory.size: invalid memory index"));
        }

        self.operand_stack
            .push_back(WasmValue::I32(i32::try_from(self.mem_size()).unwrap()));

        Ok(())
    }

    fn run_memory_grow(&mut self, mem: u32) -> Result<()> {
        if mem != 0 {
            return Err(anyhow!("memory.grow: invalid memory index"));
        }

        let additional_pages = self.pop_operand_stack().as_i32();
        self.grow_mem(u32::try_from(additional_pages)?);

        self.operand_stack
            .push_back(WasmValue::I32(i32::try_from(self.mem_size()).unwrap()));

        Ok(())
    }

    fn run_i32_binop(&mut self, i32_binop: &I32Binop) -> Result<()> {
        let b = self.pop_operand_stack().as_i32();
        let a = self.pop_operand_stack().as_i32();
        let result = match i32_binop {
            I32Binop::Eq => i32::try_from(a == b)?,
            I32Binop::Ne => i32::try_from(a != b)?,
            I32Binop::LtS => i32::try_from(a < b)?,
            I32Binop::LtU => i32::try_from((a as u32) < (b as u32))?,
            I32Binop::GtS => i32::try_from(a > b)?,
            I32Binop::GtU => i32::try_from((a as u32) > (b as u32))?,
            I32Binop::LeS => i32::try_from(a <= b)?,
            I32Binop::LeU => i32::try_from((a as u32) <= (b as u32))?,
            I32Binop::GeS => i32::try_from(a >= b)?,
            I32Binop::GeU => i32::try_from((a as u32) >= (b as u32))?,
            I32Binop::Add => i32::try_from(a + b)?,
            I32Binop::Sub => i32::try_from(a - b)?,
            I32Binop::Mul => i32::try_from(a * b)?,
            I32Binop::DivS => todo!(),
            I32Binop::DivU => todo!(),
            I32Binop::RemS => todo!(),
            I32Binop::RemU => todo!(),
            I32Binop::And => i32::try_from(a & b)?,
            I32Binop::Or => i32::try_from(a | b)?,
            I32Binop::Xor => i32::try_from(a ^ b)?,
            I32Binop::Shl => i32::try_from(a.wrapping_shl((b & 0x1f) as u32))?,
            I32Binop::ShrS => todo!(),
            I32Binop::ShrU => todo!(),
            I32Binop::Rotl => todo!(),
            I32Binop::Rotr => todo!(),
        };

        self.push_operand_stack(WasmValue::I32(result));

        Ok(())
    }
}
