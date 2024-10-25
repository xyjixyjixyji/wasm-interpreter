use monoasm::*;
use monoasm_macro::monoasm;

use crate::jit::{
    regalloc::{RegWithType, Register, X86Register, REG_TEMP, REG_TEMP2},
    utils::emit_mov_reg_to_reg,
    X86JitCompiler,
};

pub(crate) enum WasmJitControlFlowType {
    Block,
    If { else_label: Option<DestLabel> },
    Loop,
}

pub(crate) struct WasmJitControlFlowFrame {
    pub(crate) control_type: WasmJitControlFlowType,
    pub(crate) expected_stack_height: usize,
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
        monoasm!(
            &mut self.jit,
        end_label:
        );
    }

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
