use anyhow::{anyhow, Result};

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use super::{interpreter::LinearMemory, WasmFunctionExecutor};
use crate::module::{
    components::FuncDecl, insts::Instructions, value_type::WasmValue, wasm_module::WasmModule,
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
        while done_exec {
            let inst = self.func.get_inst(self.pc.0);
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
                Instructions::LocalGet { local_idx } => todo!(),
                Instructions::LocalSet { local_idx } => todo!(),
                Instructions::LocalTee { local_idx } => todo!(),
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
                    self.run_memory_size(*mem)?;
                    self.inc_pc();
                }
                Instructions::MemoryGrow { mem } => {
                    self.run_memory_grow(*mem)?;
                    self.inc_pc();
                }
                Instructions::I32Const { value } => {
                    self.push_operand_stack(WasmValue::I32(*value));
                    self.inc_pc();
                }
                Instructions::F64Const { value } => {
                    self.push_operand_stack(WasmValue::F64(*value));
                    self.inc_pc();
                }
                Instructions::I32Unop(_) => todo!(),
                Instructions::I32BinOp(_) => todo!(),
                Instructions::F64Unop(_) => todo!(),
                Instructions::F64BinOp(_) => todo!(),
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
        self.operand_stack.push_back(value);
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
}
