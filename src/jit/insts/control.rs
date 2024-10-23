use monoasm::*;
use monoasm_macro::monoasm;
use wasmparser::ValType;

use crate::{
    jit::{
        regalloc::{RegWithType, Register, X64Register, REG_TEMP},
        utils::mov_reg_to_reg,
        X86JitCompiler,
    },
    module::components::FuncDecl,
};

impl X86JitCompiler {
    pub(crate) fn compile_call(&mut self, callee_func: &FuncDecl, callee: DestLabel) {
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
                        movq R(REG_TEMP.as_index()), xmm(r.as_index());
                        pushq R(REG_TEMP.as_index());
                    );
                }
                Register::Stack(_) => panic!("stack should not be caller saved"),
            }
        }

        // setup arguments, top of the stack is the last argument
        self.setup_function_call_arguments(callee_func);

        // call the callee! and move the return value into the stack
        monoasm!(
            &mut self.jit,
            call callee;
        );

        // note that we don't want the return value to be in caller-saved registers
        // because we will pop them later in the call sequence
        let ret = self.reg_allocator.next_not_caller_saved();
        mov_reg_to_reg(&mut self.jit, ret.reg, Register::Reg(X64Register::Rax));

        // restore the stack spaced we used.....
        let restore_size = (std::cmp::max(6, callee_func.get_sig().params().len()) - 6) * 8;
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
    pub(crate) fn compile_select(
        &mut self,
        dst: RegWithType,
        cond: RegWithType,
        a: RegWithType,
        b: RegWithType,
    ) {
        let cond_is_zero = self.jit.label();
        let end_label = self.jit.label();
        mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), cond.reg);
        monoasm!(
            &mut self.jit,
            cmpq R(REG_TEMP.as_index()), 0;
            je cond_is_zero;
        );
        mov_reg_to_reg(&mut self.jit, dst.reg, a.reg); // cond != 0, set a
        monoasm!(
            &mut self.jit,
            jmp end_label;
        cond_is_zero: // cond == 0, set b
        );
        mov_reg_to_reg(&mut self.jit, dst.reg, b.reg);
        monoasm!(
            &mut self.jit,
        end_label:
        );
    }

    fn setup_function_call_arguments(&mut self, callee_func: &FuncDecl) {
        let params = callee_func.get_sig().params();
        let mut args = Vec::new();
        let mut to_push = Vec::new();

        // Collect all arguments from reg_allocator (stack top first)
        for _ in 0..params.len() {
            let arg = self.reg_allocator.pop();
            args.insert(0, arg);
        }

        // Now process parameters and arguments from last to first
        for (i, param) in params.iter().enumerate().rev() {
            let arg = args.pop().unwrap().reg; // Gets arguments from first to last
            if i < 6 {
                // Handle register arguments
                match param {
                    ValType::I32 => {
                        mov_reg_to_reg(
                            &mut self.jit,
                            Register::from_ith_argument(i as u32, false),
                            arg,
                        );
                    }
                    ValType::F64 => {
                        mov_reg_to_reg(
                            &mut self.jit,
                            Register::from_ith_argument(i as u32, true),
                            arg,
                        );
                    }
                    _ => unimplemented!("Invalid param type for JIT: {:?}", param),
                }
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
