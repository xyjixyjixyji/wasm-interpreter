use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use super::insts::{RegReconcileInfo, WasmJitControlFlowFrame, WasmJitControlFlowType};
use super::regalloc::{Register, X86Register, X86RegisterAllocator, REG_LOCAL_BASE, REG_TEMP};
use super::{JitLinearMemory, ValueType, WasmJitCompiler};
use crate::jit::regalloc::REG_TEMP_FP;
use crate::jit::utils::emit_mov_reg_to_reg;
use crate::module::components::FuncDecl;
use crate::module::insts::Instruction;
use crate::module::value_type::WasmValue;
use crate::module::wasm_module::WasmModule;
use crate::vm::WASM_DEFAULT_PAGE_SIZE_BYTE;

use anyhow::Result;
use debug_cell::RefCell;
use monoasm::{CodePtr, DestLabel, Disp, Imm, JitMemory, Reg, Rm, Scale};
use monoasm_macro::monoasm;
use wasmparser::ValType;

// Jit compile through abstract interpretation
pub struct X86JitCompiler<'a> {
    /// module
    pub(crate) module: Rc<RefCell<WasmModule<'a>>>,

    /// Register allocator, simply a register stack that controls what we can
    /// use in the current context
    ///
    /// TODO: refactor this to be per function code generator
    pub(crate) reg_allocator: X86RegisterAllocator,
    pub(crate) reg_reconcile_info: Vec<RegReconcileInfo>,

    /// the control flow stack for branching
    ///
    /// TODO: refactor this to be per function code generator
    pub(crate) control_flow_stack: VecDeque<WasmJitControlFlowFrame>,

    /// The branch table non-default target labels, for each br_table instruction,
    /// we store the non-default target labels in a vector
    ///
    /// key: func_index
    /// value: a vector of br_table instructions' non-default target labels
    pub(crate) brtable_nondefault_target_labels: HashMap<usize, Vec<Vec<DestLabel>>>,
    /// The branch table targets, for each br_table instruction, we store the
    /// target's address in a vector, so that we can access this memory after
    /// the jit relocation
    ///
    /// key: func_index
    /// value: a vector of br_table instructions' target addresses
    pub(crate) brtable_nondefault_target_addrs: HashMap<usize, Vec<Vec<u64>>>,

    /// In memory assembler
    pub(crate) jit: JitMemory,

    /// Linear memory
    pub(crate) linear_mem: JitLinearMemory,

    /// table stores functions or expressions
    ///
    /// we store the table_len separately to get the table size to make sure
    /// the table index is valid on call_indirect, when it is uninitialized,
    /// we trap
    pub(crate) tables: Vec<Vec<u32>>,
    pub(crate) table_len: Vec<usize>,

    /// global variables
    ///
    /// we separate the type from the value to get a more
    /// consistent memory layout so that we can get the global's value in asm
    /// more easily
    pub(crate) globals: Vec<u64>,
    pub(crate) global_types: Vec<ValueType>, // used statically for type checking

    /// Trap entry label
    pub(crate) trap_label: DestLabel,

    /// function labels
    pub(crate) func_labels: Vec<DestLabel>,
    pub(crate) func_addrs: Vec<u64>,       // after relocation
    pub(crate) func_sig_indices: Vec<u32>, // for call_indirect dynamic type checking
}

impl<'a> X86JitCompiler<'a> {
    pub fn new(module: Rc<RefCell<WasmModule<'a>>>) -> Self {
        let mut jit = JitMemory::new();
        let trap_label = jit.label();

        // get some statically known information
        let module = Rc::clone(&module);
        let nglobals = module.borrow().get_globals().len();
        let global_types: Vec<ValueType> = module
            .borrow()
            .get_globals()
            .iter()
            .map(|g| g.get_ty().content_type)
            .map(|ty| match ty {
                ValType::I32 => ValueType::I32,
                ValType::F64 => ValueType::F64,
                _ => unreachable!(),
            })
            .collect();
        let ntables = module.borrow().get_tables().len();
        let nfuncs = module.borrow().get_funcs().len();
        let func_sig_indices: Vec<u32> = module
            .borrow()
            .get_funcs()
            .iter()
            .map(|f| module.borrow().get_sig_index(f.get_sig()).unwrap() as u32)
            .collect();
        let func_labels = module
            .borrow()
            .get_funcs()
            .iter()
            .map(|_| jit.label())
            .collect::<Vec<_>>();
        let mem_limit = match module.borrow().get_memory() {
            Some(mem) => mem.maximum.unwrap_or(mem.initial),
            None => 0,
        };

        let mut compiler = Self {
            module,
            reg_allocator: X86RegisterAllocator::new(),
            reg_reconcile_info: Vec::new(),
            control_flow_stack: VecDeque::new(),
            jit,
            brtable_nondefault_target_labels: HashMap::new(),
            brtable_nondefault_target_addrs: HashMap::new(),
            linear_mem: JitLinearMemory::new(mem_limit),
            tables: vec![vec![]; ntables],
            table_len: vec![0; ntables],
            globals: vec![0; nglobals],
            global_types,
            trap_label,
            func_labels,
            func_addrs: vec![0; nfuncs], // setup after compilation
            func_sig_indices,
        };

        compiler.set_brtable_nondefault_target_labels();
        compiler.presize_brtable_nondefault_target_addrs();

        compiler
    }
}

impl WasmJitCompiler for X86JitCompiler<'_> {
    fn compile(&mut self, main_params: Vec<WasmValue>) -> Result<CodePtr> {
        let vm_entry_label = self.setup_runtime(main_params);

        self.compile_functions()?;

        let codeptr = self.finalize(vm_entry_label);

        log::debug!("\n{}", self.jit.dump_code().unwrap());
        Ok(unsafe { std::mem::transmute::<u64, CodePtr>(codeptr) })
    }
}

impl X86JitCompiler<'_> {
    fn compile_func(&mut self, fdecl: &FuncDecl) -> Result<()> {
        let func_index = self.module.borrow().get_func_index(fdecl).unwrap();
        let func_start = *self.func_labels.get(func_index).unwrap();
        let stack_size = self.get_stack_size_in_byte(fdecl);

        // reset per function state
        self.reg_allocator.reset();
        self.control_flow_stack.clear();
        self.reg_reconcile_info.clear();

        let end_labels = self.pregen_labals_for_ends(fdecl.get_insts());
        let else_labels = self.pregen_labels_for_else(fdecl.get_insts());
        let func_end = *end_labels.get(&(fdecl.get_insts().len() - 1)).unwrap();
        self.push_initial_control_frame(fdecl, func_start, func_end);

        // start compilation
        self.prologue(func_start, stack_size);

        let local_types = self.setup_locals(fdecl);
        self.emit_asm(
            func_index as u32,
            fdecl.get_insts(),
            &local_types,
            stack_size,
            else_labels,
            end_labels,
        )?;

        // emit return, epilogue embedded
        self.emit_function_return(Some(func_end), stack_size);

        Ok(())
    }
}

impl X86JitCompiler<'_> {
    fn setup_runtime(&mut self, main_params: Vec<WasmValue>) -> DestLabel {
        self.setup_trap_entry();
        self.setup_tables();
        self.setup_globals().expect("setup globals failed");

        // setup vm entry, the entry point of the whole program
        let module = Rc::clone(&self.module);
        let main_label = self
            .func_labels
            .get(module.borrow().get_main_index().unwrap() as usize)
            .unwrap();
        let initial_mem_size_in_byte = module
            .borrow()
            .get_memory()
            .map(|m| m.initial as usize * WASM_DEFAULT_PAGE_SIZE_BYTE)
            .unwrap_or(0) as u64;
        self.setup_vm_entry(*main_label, initial_mem_size_in_byte, main_params)
    }

    fn compile_functions(&mut self) -> Result<()> {
        let module = Rc::clone(&self.module);
        for fdecl in module.borrow().get_funcs().iter() {
            self.compile_func(fdecl)?;
        }

        Ok(())
    }

    fn finalize(&mut self, vm_entry_label: DestLabel) -> u64 {
        self.jit.finalize();

        // fill in the relocated addresses after jit relocation
        for (i, label) in self.func_labels.iter().enumerate() {
            self.func_addrs[i] = self.jit.get_label_u64(*label);
        }
        for (func_index, nondefault_target_labels) in self.brtable_nondefault_target_labels.iter() {
            let nondefault_target_addrs_ref = self
                .brtable_nondefault_target_addrs
                .get_mut(func_index)
                .unwrap();
            for (ith_table, labels) in nondefault_target_labels.iter().enumerate() {
                for (i, label) in labels.iter().enumerate() {
                    nondefault_target_addrs_ref[ith_table][i] = self.jit.get_label_u64(*label);
                }
            }
        }

        // return vm_entry address for initial execution
        self.jit.get_label_u64(vm_entry_label)
    }

    fn setup_trap_entry(&mut self) -> DestLabel {
        let trap_label = self.trap_label;
        monoasm!(
            &mut self.jit,
            trap_label:
                movq rax, 0;
                movq [rax], 1;
        );

        trap_label
    }

    fn setup_vm_entry(
        &mut self,
        main_label: DestLabel,
        initial_mem_size_in_byte: u64,
        main_params: Vec<WasmValue>,
    ) -> DestLabel {
        let vm_entry_label = self.jit.label();
        monoasm!(
            &mut self.jit,
            vm_entry_label:
        );

        // setup linear memory info
        self.linear_mem
            .init_size(&mut self.jit, initial_mem_size_in_byte);

        self.setup_data().expect("setup data segment failed");

        // setup main params
        for (i, param) in main_params.iter().enumerate() {
            if i < 6 {
                let reg = Register::from_ith_argument(i as u32);
                match param {
                    WasmValue::I32(v) => {
                        self.emit_mov_rawvalue_to_reg(*v as u64, reg);
                    }
                    WasmValue::F64(v) => {
                        self.emit_mov_rawvalue_to_reg(v.to_bits(), reg);
                    }
                }
            } else {
                // push the constant to stack
                match param {
                    WasmValue::I32(v) => {
                        self.emit_mov_rawvalue_to_reg(*v as u64, Register::Reg(REG_TEMP));
                        monoasm!(
                            &mut self.jit,
                            pushq R(REG_TEMP.as_index());
                        );
                    }
                    WasmValue::F64(v) => {
                        self.emit_mov_rawvalue_to_reg(v.to_bits(), Register::FpReg(REG_TEMP_FP));
                        monoasm!(
                            &mut self.jit,
                            pushq R(REG_TEMP_FP.as_index());
                        );
                    }
                }
            }
        }

        // jump to main
        self.emit_jmp(main_label);

        vm_entry_label
    }

    fn push_initial_control_frame(
        &mut self,
        fdecl: &FuncDecl,
        start_label: DestLabel,
        end_label: DestLabel,
    ) {
        self.control_flow_stack.push_back(WasmJitControlFlowFrame {
            control_type: WasmJitControlFlowType::Block,
            expected_stack_height: 0,
            entry_regalloc_snapshot: self.reg_allocator.clone(),
            num_results: fdecl.get_sig().results().len(),
            start_label,
            end_label,
        });
    }

    // TODO: refactor this......
    fn setup_locals(&mut self, fdecl: &FuncDecl) -> Vec<ValueType> {
        let mut local_types = Vec::new();
        let mut local_base_set = false;
        for (i, params) in fdecl.get_sig().params().iter().enumerate() {
            let r = self.reg_allocator.new_spill(ValueType::I32);

            if !local_base_set {
                // store the first local to the base of the locals
                match r.reg {
                    Register::Stack(o) => {
                        monoasm!(
                            &mut self.jit,
                            movq R(REG_LOCAL_BASE.as_index()), rbp;
                            subq R(REG_LOCAL_BASE.as_index()), (o);
                        );
                    }
                    _ => unreachable!("locals are all spilled"),
                }
                local_base_set = true;
            }

            if i < 6 {
                emit_mov_reg_to_reg(&mut self.jit, r.reg, Register::from_ith_argument(i as u32));
                match params {
                    ValType::I32 => {
                        local_types.push(ValueType::I32);
                    }
                    ValType::F64 => {
                        local_types.push(ValueType::F64);
                    }
                    _ => unreachable!(),
                }
            } else {
                // the locals are spilled to the stack
                match params {
                    ValType::I32 => {
                        monoasm!(
                            &mut self.jit,
                            movq R(REG_TEMP.as_index()), [rbp + ((i as i32 - 6) * 8 + 16)];
                        );
                        emit_mov_reg_to_reg(&mut self.jit, r.reg, Register::Reg(REG_TEMP));
                        local_types.push(ValueType::I32);
                    }
                    ValType::F64 => {
                        monoasm!(
                            &mut self.jit,
                            movsd xmm(REG_TEMP_FP.as_index()), [rbp + ((i as i32 - 6) * 8 + 16)];
                        );
                        emit_mov_reg_to_reg(&mut self.jit, r.reg, Register::FpReg(REG_TEMP_FP));
                        local_types.push(ValueType::F64);
                    }
                    _ => unreachable!(),
                }
            }
        }

        for l in fdecl.get_pure_locals() {
            let r = self.reg_allocator.new_spill(ValueType::I32);
            self.emit_mov_rawvalue_to_reg(0, r.reg);

            if !local_base_set {
                match r.reg {
                    Register::Stack(o) => {
                        monoasm!(
                            &mut self.jit,
                            movq R(REG_LOCAL_BASE.as_index()), rbp;
                            subq R(REG_LOCAL_BASE.as_index()), (o);
                        );
                    }
                    _ => unreachable!(),
                }
            }

            match l {
                ValType::I32 => local_types.push(ValueType::I32),
                ValType::F64 => local_types.push(ValueType::F64),
                _ => unreachable!(),
            }
        }

        // clear the register vector
        self.reg_allocator.clear_vec();

        local_types
    }

    fn prologue(&mut self, func_begin_label: DestLabel, stack_size: u64) {
        // NOTE: on x86-64 linux, xmms are temporary registers
        // so we don't need to save and restore them
        monoasm!(
            &mut self.jit,
        func_begin_label:
            pushq rbp;
            movq rbp, rsp;
            subq rsp, (stack_size);
            pushq rbx;
            pushq r12;
            pushq r13;
            pushq r14;
            pushq r15;
        );
    }

    fn epilogue(&mut self, stack_size: u64) {
        // NOTE: on x86-64 linux, xmms are temporary registers
        // so we don't need to save and restore them
        monoasm!(
            &mut self.jit,
            popq r15;
            popq r14;
            popq r13;
            popq r12;
            popq rbx;
            addq rsp, (stack_size);
            popq rbp;
        );
    }

    fn emit_mov_stack_top_return_reg(&mut self) {
        let stack_top = self.reg_allocator.top();
        if let Some(stack_top) = stack_top {
            emit_mov_reg_to_reg(
                &mut self.jit,
                Register::Reg(X86Register::Rax),
                stack_top.reg,
            );
        }
    }

    pub(crate) fn emit_function_return(&mut self, end_label: Option<DestLabel>, stack_size: u64) {
        if let Some(end_label) = end_label {
            self.emit_single_label(end_label);
        }

        self.emit_mov_stack_top_return_reg();
        self.epilogue(stack_size);
        monoasm!(
            &mut self.jit,
            ret;
        );
    }

    fn set_brtable_nondefault_target_labels(&mut self) {
        let mut brtable_nondefault_target_labels = HashMap::new();
        for (i, fdecl) in self.module.borrow().get_funcs().iter().enumerate() {
            let mut nondefault_target_labels = Vec::new();
            for inst in fdecl.get_insts() {
                if let Instruction::BrTable { table } = inst {
                    let mut nondefault_targets = Vec::new();
                    for _ in 0..table.targets.len() {
                        nondefault_targets.push(self.jit.label());
                    }
                    nondefault_target_labels.push(nondefault_targets);
                }
            }
            brtable_nondefault_target_labels.insert(i, nondefault_target_labels);
        }
        self.brtable_nondefault_target_labels = brtable_nondefault_target_labels;
    }

    /// setup the correct size for the brtable_nondefault_target_addrs so there
    /// will be no reallocation during the execution
    fn presize_brtable_nondefault_target_addrs(&mut self) {
        let mut brtable_nondefault_target_addrs = HashMap::new();
        for (func_index, nondefault_target_labels) in self.brtable_nondefault_target_labels.iter() {
            let mut nondefault_target_addrs = vec![vec![]; nondefault_target_labels.len()];
            for i in 0..nondefault_target_labels.len() {
                nondefault_target_addrs[i] = vec![0; nondefault_target_labels[i].len()];
            }
            brtable_nondefault_target_addrs.insert(*func_index, nondefault_target_addrs);
        }
        self.brtable_nondefault_target_addrs = brtable_nondefault_target_addrs;
    }

    fn pregen_labals_for_ends(&mut self, insts: &[Instruction]) -> HashMap<usize, DestLabel> {
        let mut end_labals = HashMap::new();
        for (i, inst) in insts.iter().enumerate() {
            if let Instruction::End = inst {
                end_labals.insert(i, self.jit.label());
            }
        }
        end_labals
    }

    fn pregen_labels_for_else(&mut self, insts: &[Instruction]) -> HashMap<usize, DestLabel> {
        let mut else_labels = HashMap::new();
        for (i, inst) in insts.iter().enumerate() {
            if let Instruction::Else = inst {
                else_labels.insert(i, self.jit.label());
            }
        }
        else_labels
    }
}
