use std::rc::Rc;

use super::regalloc::{Register, X86Register, X86RegisterAllocator, REG_LOCAL_BASE, REG_TEMP};
use super::{JitLinearMemory, ValueType, WasmJitCompiler};
use crate::jit::regalloc::REG_TEMP_FP;
use crate::jit::utils::emit_mov_reg_to_reg;
use crate::module::components::FuncDecl;
use crate::module::insts::Instruction;
use crate::module::value_type::WasmValue;
use crate::module::wasm_module::WasmModule;

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
    pub(crate) reg_allocator: X86RegisterAllocator,

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

        let mut compiler = Self {
            module,
            reg_allocator: X86RegisterAllocator::new(),
            jit,
            linear_mem: JitLinearMemory::new(),
            tables: vec![vec![]; ntables],
            table_len: vec![0; ntables],
            globals: vec![0; nglobals],
            global_types,
            trap_label,
            func_labels: vec![],
            func_addrs: vec![0; nfuncs],
            func_sig_indices,
        };

        compiler.setup_trap_entry();

        compiler
    }
}

impl WasmJitCompiler for X86JitCompiler<'_> {
    fn compile(
        &mut self,
        initial_mem_size_in_byte: u64,
        main_params: Vec<WasmValue>,
    ) -> Result<CodePtr> {
        self.func_labels = self
            .module
            .borrow()
            .get_funcs()
            .iter()
            .map(|_| self.jit.label())
            .collect();

        // TODO: setup globals
        self.setup_tables();

        // setup vm_entry
        let main_index = self.module.borrow().get_main_index().unwrap();
        let main_label = self.func_labels.get(main_index as usize).unwrap();
        let vm_entry_label =
            self.setup_vm_entry(*main_label, initial_mem_size_in_byte, main_params);

        // compile all functions
        let module = Rc::clone(&self.module);
        for fdecl in module.borrow().get_funcs().iter() {
            self.compile_func(fdecl)?;
        }

        self.jit.finalize();
        for (i, label) in self.func_labels.iter().enumerate() {
            self.func_addrs[i] = self.jit.get_label_u64(*label);
        }

        // return vm_entry address for initial execution
        let codeptr = self.jit.get_label_u64(vm_entry_label);

        log::debug!("\n{}", self.jit.dump_code().unwrap());
        Ok(unsafe { std::mem::transmute::<u64, CodePtr>(codeptr) })
    }
}

impl X86JitCompiler<'_> {
    fn compile_func(&mut self, fdecl: &FuncDecl) -> Result<()> {
        let func_index = self.module.borrow().get_func_index(fdecl).unwrap();
        let func_begin_label = *self.func_labels.get(func_index).unwrap();
        let stack_size = self.get_stack_size_in_byte(fdecl);
        self.reg_allocator.reset();

        // start compilation
        monoasm!(
            &mut self.jit,
            func_begin_label:
        );
        self.prologue(stack_size);

        let local_types = self.setup_locals(fdecl);
        self.emit_asm(fdecl.get_insts(), &local_types)?;
        // return...
        let stack_top = self.reg_allocator.top();
        if let Some(stack_top) = stack_top {
            emit_mov_reg_to_reg(
                &mut self.jit,
                Register::Reg(X86Register::Rax),
                stack_top.reg,
            );
        }

        self.epilogue(stack_size);
        monoasm!(
            &mut self.jit,
            ret;
        );

        Ok(())
    }
}

impl X86JitCompiler<'_> {
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
                        self.emit_mov_i32_to_reg(*v, reg);
                    }
                    WasmValue::F64(v) => {
                        self.emit_mov_f64_to_reg(*v, reg);
                    }
                }
            } else {
                // push the constant to stack
                match param {
                    WasmValue::I32(v) => {
                        self.emit_mov_i32_to_reg(*v, Register::Reg(REG_TEMP));
                        monoasm!(
                            &mut self.jit,
                            pushq R(REG_TEMP.as_index());
                        );
                    }
                    WasmValue::F64(v) => {
                        self.emit_mov_f64_to_reg(*v, Register::FpReg(REG_TEMP_FP));
                        monoasm!(
                            &mut self.jit,
                            pushq R(REG_TEMP_FP.as_index());
                        );
                    }
                }
            }
        }

        // jump to main
        monoasm!(
            &mut self.jit,
            jmp main_label;
        );

        vm_entry_label
    }

    fn setup_locals(&mut self, fdecl: &FuncDecl) -> Vec<ValueType> {
        let mut local_types = Vec::new();
        for (i, params) in fdecl.get_sig().params().iter().enumerate() {
            let r = self.reg_allocator.new_spill(ValueType::I32);

            if i == 0 {
                // store the first local to the base of the locals
                match r.reg {
                    Register::Stack(o) => {
                        monoasm!(
                            &mut self.jit,
                            movq R(REG_LOCAL_BASE.as_index()), rsp;
                            addq R(REG_LOCAL_BASE.as_index()), (o);
                        );
                    }
                    _ => unreachable!("locals are all spilled"),
                }
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
                            movq R(REG_TEMP.as_index()), [rbp + ((i as i32 - 6) * 8 + 8)];
                        );
                        emit_mov_reg_to_reg(&mut self.jit, r.reg, Register::Reg(REG_TEMP));
                        local_types.push(ValueType::I32);
                    }
                    ValType::F64 => {
                        monoasm!(
                            &mut self.jit,
                            movsd xmm(REG_TEMP_FP.as_index()), [rbp + ((i as i32 - 6) * 8 + 8)];
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
            self.emit_mov_i32_to_reg(0, r.reg);

            match l {
                (_, ValType::I32) => local_types.push(ValueType::I32),
                (_, ValType::F64) => local_types.push(ValueType::F64),
                _ => unreachable!(),
            }
        }

        // clear the register vector
        self.reg_allocator.clear_vec();

        local_types
    }

    fn prologue(&mut self, stack_size: u64) {
        // NOTE: on x86-64 linux, xmms are temporary registers
        // so we don't need to save and restore them
        monoasm!(
            &mut self.jit,
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
}

impl X86JitCompiler<'_> {
    // Get the stack size usage of the function, used for stack allocation
    // We get only an upper bound approximate, since we don't want too much overhead
    fn get_stack_size_in_byte(&self, fdecl: &FuncDecl) -> u64 {
        let nlocals = (fdecl.get_pure_locals().len() + fdecl.get_sig().params().len()) as u64;

        let mut max_stack_depth: u64 = 0;
        let mut current_stack_depth: u64 = 0;
        let mut block_stack = Vec::new();

        let insts = fdecl.get_insts();
        let mut pc = 0;
        while pc < insts.len() {
            let inst = &insts[pc];
            match inst {
                // Constants push a value onto the stack
                Instruction::I32Const { .. } | Instruction::F64Const { .. } => {
                    current_stack_depth += 1;
                }

                // Unreachable instruction; for approximation, reset stack depth
                Instruction::Unreachable => {
                    current_stack_depth = 0;
                }

                // No operation; stack depth remains the same
                Instruction::Nop => {}

                // Drop pops one value from the stack
                Instruction::Drop => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                }

                // Binary operations pop two values and push one; net effect is -1
                Instruction::I32Binop(_) | Instruction::F64Binop(_) => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                }

                // Unary operations consume one value and produce one; net effect is 0
                Instruction::I32Unop(_) | Instruction::F64Unop(_) => {}

                // Block, Loop, If: push current stack depth onto block stack
                Instruction::Block { .. } | Instruction::Loop { .. } | Instruction::If { .. } => {
                    block_stack.push(current_stack_depth);
                }

                // Else: reset stack depth to the depth at the start of the block
                Instruction::Else => {
                    if let Some(depth_at_if) = block_stack.last().cloned() {
                        current_stack_depth = depth_at_if;
                    }
                }

                // End: pop from block stack and take the maximum of current and block start depth
                Instruction::End => {
                    if let Some(depth_at_block_start) = block_stack.pop() {
                        current_stack_depth =
                            std::cmp::max(current_stack_depth, depth_at_block_start);
                    }
                }

                // Branch instructions; for approximation, we can reset or leave the stack depth
                Instruction::Br { .. } => {
                    // For simplicity, we'll leave the stack depth unchanged
                }

                // BrIf pops one value (the condition)
                Instruction::BrIf { .. } => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                    // Stack depth after branch remains the same for upper bound
                }

                // BrTable pops one value (the index)
                Instruction::BrTable { .. } => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                    // Stack depth remains unchanged for approximation
                }

                // Function calls; assume stack depth remains the same for upper bound
                Instruction::Call { .. } | Instruction::CallIndirect { .. } => {
                    // If you have type info, adjust current_stack_depth accordingly
                }

                // Return resets the current stack depth
                Instruction::Return => {
                    current_stack_depth = 0;
                }

                // Select pops three values and pushes one; net effect is -2
                Instruction::Select => {
                    current_stack_depth = current_stack_depth.saturating_sub(2);
                }

                // LocalGet pushes a value onto the stack
                Instruction::LocalGet { .. } => {
                    current_stack_depth += 1;
                }

                // LocalSet pops one value from the stack
                Instruction::LocalSet { .. } => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                }

                // LocalTee pops and then pushes the same value; net effect is 0
                Instruction::LocalTee { .. } => {}

                // GlobalGet pushes a value onto the stack
                Instruction::GlobalGet { .. } => {
                    current_stack_depth += 1;
                }

                // GlobalSet pops one value from the stack
                Instruction::GlobalSet { .. } => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                }

                // Memory load instructions pop one address and push one value; net effect is 0
                Instruction::I32Load { .. }
                | Instruction::F64Load { .. }
                | Instruction::I32Load8S { .. }
                | Instruction::I32Load8U { .. }
                | Instruction::I32Load16S { .. }
                | Instruction::I32Load16U { .. } => {
                    // Pops one, pushes one; stack depth remains the same
                }

                // Memory store instructions pop two values (value and address); net effect is -2
                Instruction::I32Store { .. }
                | Instruction::F64Store { .. }
                | Instruction::I32Store8 { .. }
                | Instruction::I32Store16 { .. } => {
                    if current_stack_depth >= 2 {
                        current_stack_depth -= 2;
                    } else {
                        current_stack_depth = 0;
                    }
                }

                // MemorySize pushes one value onto the stack
                Instruction::MemorySize { .. } => {
                    current_stack_depth += 1;
                }

                // MemoryGrow pops one and pushes one; net effect is 0
                Instruction::MemoryGrow { .. } => {}
            }

            // Update max_stack_depth if current_stack_depth exceeds it
            if current_stack_depth > max_stack_depth {
                max_stack_depth = current_stack_depth;
            }

            pc += 1;
        }

        // Calculate total stack size: locals + max stack depth
        // Each stack slot is 8 bytes (for alignment)
        //
        // +1 for storing the current memory size
        let total_stack_size = (nlocals + max_stack_depth + 1) * 8;

        // Align stack size to 16 bytes (common requirement for x86-64)
        (total_stack_size + 15) & !15
    }
}
