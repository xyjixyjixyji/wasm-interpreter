use crate::{
    jit::{
        regalloc::{
            RegWithType, Register, X86Register, REG_TEMP, REG_TEMP2, REG_TEMP_FP, REG_TEMP_FP2,
        },
        utils::emit_mov_reg_to_reg,
        ValueType, X86JitCompiler,
    },
    module::insts::{F64Binop, I32Binop},
};

use monoasm::*;
use monoasm_macro::monoasm;

impl X86JitCompiler<'_> {
    // jit compile *a = a op b*
    pub(crate) fn emit_f64_binop(&mut self, binop: &F64Binop) {
        let b = self.reg_allocator.pop().reg;
        let a = self.reg_allocator.pop().reg;

        emit_mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP), a);
        emit_mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP2), b);

        match binop {
            F64Binop::Add => {
                monoasm!(
                    &mut self.jit,
                    addsd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
            F64Binop::Eq => todo!(),
            F64Binop::Ne => todo!(),
            F64Binop::Lt => todo!(),
            F64Binop::Gt => todo!(),
            F64Binop::Le => todo!(),
            F64Binop::Ge => todo!(),
            F64Binop::Sub => todo!(),
            F64Binop::Mul => todo!(),
            F64Binop::Div => todo!(),
            F64Binop::Min => todo!(),
            F64Binop::Max => todo!(),
        }

        emit_mov_reg_to_reg(&mut self.jit, a, Register::FpReg(REG_TEMP_FP));
        self.reg_allocator.push(RegWithType::new(a, ValueType::F64));
    }

    pub(crate) fn emit_i32_binop(&mut self, binop: &I32Binop) {
        let b = self.reg_allocator.pop();
        let a = self.reg_allocator.pop();

        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), a.reg);
        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP2), b.reg);

        match binop {
            I32Binop::Eq => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    seteq R(REG_TEMP.as_index()); // a = a == b
                );
            }
            I32Binop::Ne => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setne R(REG_TEMP.as_index()); // a = a != b
                );
            }
            I32Binop::LtS => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    sets R(REG_TEMP.as_index()); // a = a < b
                );
            }
            I32Binop::LtU => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setb R(REG_TEMP.as_index()); // a = a < b
                );
            }
            I32Binop::GtS => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setgt R(REG_TEMP.as_index()); // a = a > b
                );
            }
            I32Binop::GtU => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    seta R(REG_TEMP.as_index()); // a = a > b
                );
            }
            I32Binop::LeS => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setle R(REG_TEMP.as_index()); // a = a <= b
                );
            }
            I32Binop::LeU => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setbe R(REG_TEMP.as_index()); // a = a <= b
                );
            }
            I32Binop::GeS => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setge R(REG_TEMP.as_index()); // a = a >= b
                );
            }
            I32Binop::GeU => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setae R(REG_TEMP.as_index()); // a = a >= b
                );
            }
            I32Binop::Add => {
                monoasm!(
                    &mut self.jit,
                    addq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a + b
                );
            }
            I32Binop::Sub => {
                monoasm!(
                    &mut self.jit,
                    subq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a - b
                );
            }
            I32Binop::Mul => {
                monoasm!(
                    &mut self.jit,
                    imul R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a * b
                );
            }
            I32Binop::DivS | I32Binop::RemS => {
                let trap_label = self.trap_label;
                let ok_label = self.jit.label();
                monoasm!(
                    &mut self.jit,
                    // div by zero check
                    testq R(REG_TEMP2.as_index()), R(REG_TEMP2.as_index());
                    jz trap_label;

                    // overflow check
                    pushq R(X86Register::Rax.as_index());
                    pushq R(X86Register::Rdx.as_index());
                    movq R(X86Register::Rax.as_index()), (0xFFFFFFFF80000000);
                    cmpq R(REG_TEMP.as_index()), R(X86Register::Rax.as_index());
                    jne ok_label;
                    movq R(X86Register::Rax.as_index()), (0xFFFFFFFFFFFFFFFF);
                    cmpq R(REG_TEMP2.as_index()), R(X86Register::Rax.as_index());
                    jne ok_label;
                    jmp trap_label;

                ok_label:
                    movq R(X86Register::Rax.as_index()), R(REG_TEMP.as_index());
                    cqo; // RDX:RAX
                    idiv R(REG_TEMP2.as_index()); // RAX: quotient, RDX: remainder
                );
                if matches!(binop, I32Binop::DivS) {
                    emit_mov_reg_to_reg(
                        &mut self.jit,
                        Register::Reg(REG_TEMP),
                        Register::Reg(X86Register::Rax),
                    );
                } else {
                    emit_mov_reg_to_reg(
                        &mut self.jit,
                        Register::Reg(REG_TEMP),
                        Register::Reg(X86Register::Rdx),
                    );
                }
                monoasm!(
                    &mut self.jit,
                    popq R(X86Register::Rdx.as_index());
                    popq R(X86Register::Rax.as_index());
                );
            }
            I32Binop::DivU | I32Binop::RemU => {
                let trap_label = self.trap_label;
                let ok_label = self.jit.label();
                monoasm!(
                    &mut self.jit,
                    // div by zero check
                    testq R(REG_TEMP2.as_index()), R(REG_TEMP2.as_index());
                    jz trap_label;

                ok_label:
                    pushq R(X86Register::Rax.as_index());
                    pushq R(X86Register::Rdx.as_index());

                    // Clear RDX (for unsigned division, RDX should be 0)
                    xorq R(X86Register::Rdx.as_index()), R(X86Register::Rdx.as_index());

                    // Move dividend into RAX
                    movq R(X86Register::Rax.as_index()), R(REG_TEMP.as_index());

                    // Perform the unsigned division
                    div R(REG_TEMP2.as_index()); // RAX: quotient, RDX: remainder
                );
                if matches!(binop, I32Binop::DivU) {
                    emit_mov_reg_to_reg(
                        &mut self.jit,
                        Register::Reg(REG_TEMP),
                        Register::Reg(X86Register::Rax),
                    );
                } else {
                    emit_mov_reg_to_reg(
                        &mut self.jit,
                        Register::Reg(REG_TEMP),
                        Register::Reg(X86Register::Rdx),
                    );
                }
                monoasm!(
                    &mut self.jit,
                    popq R(X86Register::Rdx.as_index());
                    popq R(X86Register::Rax.as_index());
                );
            }
            I32Binop::And => {
                monoasm!(
                    &mut self.jit,
                    andq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a & b
                );
            }
            I32Binop::Or => {
                monoasm!(
                    &mut self.jit,
                    orq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a | b
                );
            }
            I32Binop::Xor => {
                monoasm!(
                    &mut self.jit,
                    xorq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a ^ b
                );
            }
            I32Binop::Shl => todo!(),
            I32Binop::ShrS => todo!(),
            I32Binop::ShrU => todo!(),
            I32Binop::Rotl => todo!(),
            I32Binop::Rotr => todo!(),
        }

        emit_mov_reg_to_reg(&mut self.jit, a.reg, Register::Reg(REG_TEMP));
        self.reg_allocator.push(a);
    }
}
