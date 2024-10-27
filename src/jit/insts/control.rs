use monoasm::*;
use monoasm_macro::monoasm;
use wasmparser::BlockType;

use crate::{
    jit::{
        regalloc::{RegWithType, Register, X86Register, X86RegisterAllocator, REG_TEMP, REG_TEMP2},
        utils::emit_mov_reg_to_reg,
        X86JitCompiler,
    },
    module::insts::BrTable,
    vm::{block_type_num_results, stack_height_delta},
};

#[derive(Debug, Clone)]
pub(crate) enum WasmJitControlFlowType {
    Block,
    If,
    Loop,
}

#[derive(Debug, Clone)]
pub(crate) struct WasmJitControlFlowFrame {
    pub(crate) control_type: WasmJitControlFlowType,
    pub(crate) expected_stack_height: usize,
    pub(crate) entry_regalloc_snapshot: X86RegisterAllocator,
    pub(crate) num_results: usize,
    pub(crate) start_label: DestLabel,
    pub(crate) end_label: DestLabel,
}

impl X86JitCompiler<'_> {
    /// compile the call_indirect instruction
    /// we get the callee label and emit the call instruction sequence
    pub(crate) fn emit_call_indirect(
        &mut self,
        callee_index_in_table: Register,
        type_index: u32,
        table_index: u32,
    ) {
        // get the callee label by reading the table
        let nr_args = self
            .module
            .borrow()
            .get_sig(type_index)
            .unwrap()
            .params()
            .len();

        emit_mov_reg_to_reg(
            &mut self.jit,
            Register::Reg(REG_TEMP2),
            callee_index_in_table,
        ); // reg_temp2 = ind

        // compare the table index with the number of elements in the table
        // if it's greater than the number of elements, we should trap
        let table_size = *self.table_len.get(table_index as usize).unwrap();
        let trap_label = self.trap_label;
        monoasm!(
            &mut self.jit,
            cmpq R(REG_TEMP2.as_index()), (table_size);
            jge trap_label;
            cmpq R(REG_TEMP2.as_index()), 0;
            js trap_label; // negative index
        );

        // dynamic type checking for signature match
        let func_sig_indices = self.func_sig_indices.as_ptr();
        monoasm!(
            &mut self.jit,
            movq R(REG_TEMP.as_index()), (func_sig_indices);
            movl R(REG_TEMP.as_index()), [R(REG_TEMP.as_index()) + R(REG_TEMP2.as_index()) * 4]; // reg_temp = func_sig_index
            cmpq R(REG_TEMP.as_index()), (type_index);
            jne trap_label;
        );

        let table_data = self.tables.get(table_index as usize).unwrap().as_ptr();
        monoasm!(
            &mut self.jit,
            movq R(REG_TEMP.as_index()), (table_data);
            movl R(REG_TEMP.as_index()), [R(REG_TEMP.as_index()) + R(REG_TEMP2.as_index()) * 4]; // reg_temp = func_index
        );

        self.emit_call(REG_TEMP, nr_args);
    }

    pub(crate) fn emit_call(&mut self, callee_index: X86Register, nr_args: usize) {
        emit_mov_reg_to_reg(
            &mut self.jit,
            Register::Reg(REG_TEMP),
            Register::Reg(callee_index),
        ); // reg_temp = callee_index

        // save caller-saved registers
        let caller_saved_regs = self.reg_allocator.get_used_caller_saved_registers();

        for reg in &caller_saved_regs {
            match reg {
                Register::Reg(r) => {
                    monoasm!(
                        &mut self.jit,
                        pushq R(r.as_index());
                    );
                }
                Register::FpReg(r) => {
                    monoasm!(
                        &mut self.jit,
                        movq R(REG_TEMP2.as_index()), xmm(r.as_index());
                        pushq R(REG_TEMP2.as_index());
                    );
                }
                Register::Stack(_) => panic!("stack should not be caller saved"),
            }
        }

        // setup arguments, top of the stack is the last argument
        self.setup_function_call_arguments(nr_args);

        // get callee address and call it
        let func_addrs_ptr = self.func_addrs.as_ptr();
        monoasm!(
            &mut self.jit,
            movq R(REG_TEMP2.as_index()), (func_addrs_ptr);
            movq rax, [R(REG_TEMP2.as_index()) + R(REG_TEMP.as_index()) * 8];
            call rax;
        );

        // note that we don't want the return value to be in caller-saved registers
        // because we will pop them later in the call sequence
        let ret = self.reg_allocator.next_not_caller_saved();
        emit_mov_reg_to_reg(&mut self.jit, ret.reg, Register::Reg(X86Register::Rax));

        // restore the stack spaced we used.....
        let restore_size = (std::cmp::max(6, nr_args) - 6) * 8;
        monoasm!(
            &mut self.jit,
            addq rsp, (restore_size);
        );

        // restore caller-saved registers
        for reg in caller_saved_regs.iter().rev() {
            match reg {
                Register::Reg(r) => {
                    monoasm!(
                        &mut self.jit,
                        popq R(r.as_index());
                    );
                }
                Register::FpReg(r) => {
                    monoasm!(
                        &mut self.jit,
                        popq R(REG_TEMP.as_index());
                        movq xmm(r.as_index()), R(REG_TEMP.as_index());
                    );
                }
                Register::Stack(_) => panic!("stack should not be caller saved"),
            }
        }
    }

    /// compile the select instruction
    /// select cond, a, b
    /// if cond != 0, then set a to the result, otherwise set b
    pub(crate) fn emit_select(
        &mut self,
        dst: RegWithType,
        cond: RegWithType,
        a: RegWithType,
        b: RegWithType,
    ) {
        let cond_is_zero = self.jit.label();
        let end_label = self.jit.label();
        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), cond.reg);
        monoasm!(
            &mut self.jit,
            cmpq R(REG_TEMP.as_index()), 0;
            je cond_is_zero;
        );
        emit_mov_reg_to_reg(&mut self.jit, dst.reg, a.reg); // cond != 0, set a
        monoasm!(
            &mut self.jit,
            jmp end_label;
        cond_is_zero: // cond == 0, set b
        );
        emit_mov_reg_to_reg(&mut self.jit, dst.reg, b.reg);
        self.emit_single_label(end_label);
    }

    pub(crate) fn emit_block(
        &mut self,
        ty: BlockType,
        block_begin: DestLabel,
        block_end: DestLabel,
    ) {
        let expected_stack_size =
            self.reg_allocator.size() + stack_height_delta(self.module.clone(), ty);
        self.control_flow_stack.push_back(WasmJitControlFlowFrame {
            control_type: WasmJitControlFlowType::Block,
            expected_stack_height: expected_stack_size,
            entry_regalloc_snapshot: self.reg_allocator.clone(),
            num_results: block_type_num_results(self.module.clone(), ty),
            start_label: block_begin,
            end_label: block_end,
        });

        self.emit_single_label(block_begin);
    }

    pub(crate) fn emit_if(
        &mut self,
        cond: Register,
        ty: BlockType,
        else_label: Option<DestLabel>,
        end_label: DestLabel,
    ) {
        let start_label = self.jit.label();

        let expected_stack_height =
            self.reg_allocator.size() + stack_height_delta(self.module.clone(), ty);
        self.control_flow_stack.push_back(WasmJitControlFlowFrame {
            control_type: WasmJitControlFlowType::If,
            expected_stack_height,
            entry_regalloc_snapshot: self.reg_allocator.clone(),
            num_results: block_type_num_results(self.module.clone(), ty),
            start_label,
            end_label,
        });

        self.emit_single_label(start_label);
        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), cond);
        if let Some(else_label) = else_label {
            monoasm!(
                &mut self.jit,
                cmpq R(REG_TEMP.as_index()), 0;
                jz else_label; /* else block executes until it reaches end */
            );
        } else {
            // if there is no else block, we jump to the end directly
            monoasm!(
                &mut self.jit,
                cmpq R(REG_TEMP.as_index()), 0;
                jmp end_label;
            );
        }
    }

    pub(crate) fn emit_br_table(
        &mut self,
        index: Register,
        table: &BrTable,
        which_func: u32,
        which_table: usize,
    ) {
        let table_size = table.targets.len();
        let default_target_label = self.jit.label();
        let target_labels =
            self.brtable_nondefault_target_labels[&(which_func as usize)][which_table].clone();

        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), index);
        monoasm!(
            &mut self.jit,
            cmpq R(REG_TEMP.as_index()), 0;
            js default_target_label; // negative index
            cmpq R(REG_TEMP.as_index()), (table_size);
            jae default_target_label; // out of bound index
        );

        // now we are jumping to actual target inside the table
        // width = 8 because we are storing u64
        let target_addrs_ptr =
            self.brtable_nondefault_target_addrs[&(which_func as usize)][which_table].as_ptr();
        monoasm!(
            &mut self.jit,
            movq R(REG_TEMP2.as_index()), (target_addrs_ptr);
            jmp [R(REG_TEMP2.as_index()) + R(REG_TEMP.as_index()) * 8];
        );

        // construct jump table
        for (i, target) in table.targets.iter().enumerate() {
            let target_label = target_labels[i];
            self.emit_single_label(target_label);
            self.emit_br(*target);
        }

        self.emit_single_label(default_target_label);
        self.emit_br(table.default_target);
    }

    pub(crate) fn emit_br_if(&mut self, cond: Register, rel_depth: u32) {
        let skip_br = self.jit.label();
        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), cond);
        monoasm!(
            &mut self.jit,
            cmpq R(REG_TEMP.as_index()), 0;
            jz skip_br;
        );
        self.emit_br(rel_depth);
        self.emit_single_label(skip_br);
    }

    pub(crate) fn emit_br(&mut self, rel_depth: u32) {
        let target_depth = rel_depth as usize;
        let stack_depth = self.control_flow_stack.len();

        if target_depth >= stack_depth {
            panic!("invalid branch target depth");
        }

        let target_frame = self.control_flow_stack[stack_depth - target_depth - 1].clone();

        match target_frame.control_type {
            WasmJitControlFlowType::Block { .. } => {
                // we dont need to truncate the stack here, because the jit code
                // is not actually run during codegen
                self.emit_jmp(target_frame.end_label);
            }
            WasmJitControlFlowType::If { .. } => todo!(),
            WasmJitControlFlowType::Loop { .. } => todo!(),
        }
    }

    pub(crate) fn emit_single_label(&mut self, label: DestLabel) {
        monoasm!(
            &mut self.jit,
            label:
        );
    }

    pub(crate) fn emit_jmp(&mut self, dst: DestLabel) {
        monoasm!(
            &mut self.jit,
            jmp dst;
        );
    }
}

impl X86JitCompiler<'_> {
    fn setup_function_call_arguments(&mut self, nr_args: usize) {
        let mut args = Vec::new();
        let mut to_push = Vec::new();

        // Collect all arguments from reg_allocator (stack top first)
        for _ in 0..nr_args {
            let arg = self.reg_allocator.pop();
            args.insert(0, arg);
        }

        // Now process parameters and arguments from last to first
        for i in (0..nr_args).rev() {
            let arg = args.pop().unwrap().reg; // Gets arguments from first to last
            if i < 6 {
                // Handle register arguments
                emit_mov_reg_to_reg(&mut self.jit, Register::from_ith_argument(i as u32), arg);
            } else {
                to_push.push(arg);
            }
        }

        for arg in to_push.iter().rev() {
            match arg {
                Register::Reg(r) => {
                    monoasm!(
                        &mut self.jit,
                        pushq R(r.as_index());
                    );
                }
                Register::FpReg(r) => {
                    monoasm!(
                        &mut self.jit,
                        movq R(REG_TEMP.as_index()), xmm(r.as_index());
                        pushq R(REG_TEMP.as_index());
                    );
                }
                Register::Stack(o) => {
                    monoasm!(
                        &mut self.jit,
                        movq R(REG_TEMP.as_index()), [rsp + (*o)];
                        pushq R(REG_TEMP.as_index());
                    );
                }
            }
        }
    }
}
