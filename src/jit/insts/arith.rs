use crate::{
    jit::{
        regalloc::{
            RegWithType, Register, X86Register, REG_TEMP, REG_TEMP2, REG_TEMP_FP, REG_TEMP_FP2,
        },
        utils::emit_mov_reg_to_reg,
        ValueType, X86JitCompiler,
    },
    module::insts::{F64Binop, F64Unop, I32Binop, I32Unop},
};

use monoasm::*;
use monoasm_macro::monoasm;

impl X86JitCompiler<'_> {
    pub(crate) fn emit_f64_unop(&mut self, unop: &F64Unop) {
        let a = self.reg_allocator.pop_noopt().reg;
        emit_mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP), a);

        match unop {
            F64Unop::Abs => {
                self.emit_mov_rawvalue_to_reg(0x7fffffffffffffff, Register::FpReg(REG_TEMP_FP2));
                monoasm!(
                    &mut self.jit,
                    andpd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
            F64Unop::Neg => {
                self.emit_mov_rawvalue_to_reg(0x8000000000000000, Register::FpReg(REG_TEMP_FP2));
                monoasm!(
                    &mut self.jit,
                    xorpd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
            F64Unop::Ceil => {
                monoasm!(
                    &mut self.jit,
                    roundpd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP.as_index()), (0x02);
                );
            }
            F64Unop::Floor => {
                monoasm!(
                    &mut self.jit,
                    roundpd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP.as_index()), (0x01);
                );
            }
            F64Unop::Trunc => {
                monoasm!(
                    &mut self.jit,
                    roundpd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP.as_index()), (0x03);
                );
            }
            F64Unop::Nearest => {
                monoasm!(
                    &mut self.jit,
                    roundpd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP.as_index()), (0x00);
                );
            }
            F64Unop::Sqrt => todo!(),
            F64Unop::I32TruncF64S => todo!(),
            F64Unop::I32TruncF64U => todo!(),
        }

        emit_mov_reg_to_reg(&mut self.jit, a, Register::FpReg(REG_TEMP_FP));
        self.reg_allocator.push(RegWithType::new(a, ValueType::F64));
    }

    // jit compile *a = a op b*
    pub(crate) fn emit_f64_binop(&mut self, binop: &F64Binop) {
        let b = self.reg_allocator.pop_noopt().reg;
        let a = self.reg_allocator.pop_noopt().reg;

        emit_mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP), a);
        emit_mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP2), b);

        match binop {
            F64Binop::Add => {
                monoasm!(
                    &mut self.jit,
                    addsd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
            F64Binop::Eq => {
                monoasm!(
                    &mut self.jit,
                    ucomisd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    seteq R(REG_TEMP.as_index());
                );
                let dst = self.reg_allocator.next();
                emit_mov_reg_to_reg(&mut self.jit, dst.reg, Register::Reg(REG_TEMP));
                self.reg_allocator.push(dst);
                return; // this returns a i32, so we return early
            }
            F64Binop::Ne => {
                monoasm!(
                    &mut self.jit,
                    ucomisd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setne R(REG_TEMP.as_index());
                );
                let dst = self.reg_allocator.next();
                emit_mov_reg_to_reg(&mut self.jit, dst.reg, Register::Reg(REG_TEMP));
                self.reg_allocator.push(dst);
                return; // this returns a i32, so we return early
            }
            F64Binop::Lt => {
                monoasm!(
                    &mut self.jit,
                    ucomisd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setb R(REG_TEMP.as_index());
                );
                let dst = self.reg_allocator.next();
                emit_mov_reg_to_reg(&mut self.jit, dst.reg, Register::Reg(REG_TEMP));
                self.reg_allocator.push(dst);
                return; // this returns a i32, so we return early
            }
            F64Binop::Gt => {
                monoasm!(
                    &mut self.jit,
                    ucomisd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    seta R(REG_TEMP.as_index());
                );
                let dst = self.reg_allocator.next();
                emit_mov_reg_to_reg(&mut self.jit, dst.reg, Register::Reg(REG_TEMP));
                self.reg_allocator.push(dst);
                return; // this returns a i32, so we return early
            }
            F64Binop::Le => {
                monoasm!(
                    &mut self.jit,
                    ucomisd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setbe R(REG_TEMP.as_index());
                );
                let dst = self.reg_allocator.next();
                emit_mov_reg_to_reg(&mut self.jit, dst.reg, Register::Reg(REG_TEMP));
                self.reg_allocator.push(dst);
                return; // this returns a i32, so we return early
            }
            F64Binop::Ge => {
                monoasm!(
                    &mut self.jit,
                    ucomisd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setae R(REG_TEMP.as_index());
                );
                let dst = self.reg_allocator.next();
                emit_mov_reg_to_reg(&mut self.jit, dst.reg, Register::Reg(REG_TEMP));
                self.reg_allocator.push(dst);
                return; // this returns a i32, so we return early
            }
            F64Binop::Sub => {
                monoasm!(
                    &mut self.jit,
                    subsd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
            F64Binop::Mul => {
                monoasm!(
                    &mut self.jit,
                    mulsd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
            F64Binop::Div => {
                monoasm!(
                    &mut self.jit,
                    divsd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
            F64Binop::Min => {
                monoasm!(
                    &mut self.jit,
                    minsd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
            F64Binop::Max => {
                monoasm!(
                    &mut self.jit,
                    maxsd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
        }

        emit_mov_reg_to_reg(&mut self.jit, a, Register::FpReg(REG_TEMP_FP));
        self.reg_allocator.push(RegWithType::new(a, ValueType::F64));
    }

    pub(crate) fn emit_i32_unop(&mut self, unop: &I32Unop) {
        let a = self.reg_allocator.pop_noopt();
        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), a.reg);

        match unop {
            I32Unop::Eqz => todo!(),
            I32Unop::Clz => todo!(),
            I32Unop::Ctz => todo!(),
            I32Unop::Popcnt => todo!(),
            I32Unop::Extend8S => todo!(),
            I32Unop::Extend16S => todo!(),
            I32Unop::F64ConvertI32S => todo!(),
            I32Unop::F64ConvertI32U => todo!(),
        }
    }

    pub(crate) fn emit_i32_binop(&mut self, binop: &I32Binop) {
        let b = self.reg_allocator.pop_noopt();
        let a = self.reg_allocator.pop_noopt();

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
            I32Binop::Shl => {
                monoasm!(
                    &mut self.jit,
                    pushq rcx;
                    movb rcx, R(REG_TEMP2.as_index());
                    andb cl, (0x1F);
                    shlq R(REG_TEMP.as_index()), cl; // a = a << b
                    popq rcx;
                );
            }
            I32Binop::ShrS => {
                monoasm!(
                    &mut self.jit,
                    pushq rcx;
                    movb rcx, R(REG_TEMP2.as_index());
                    andb cl, (0x1F);
                    sarq R(REG_TEMP.as_index()), cl; // a = a >> b
                    popq rcx;
                );
            }
            I32Binop::ShrU => {
                monoasm!(
                    &mut self.jit,
                    pushq rcx;
                    movb rcx, R(REG_TEMP2.as_index());
                    andb cl, (0x1F);
                    movl R(REG_TEMP2.as_index()), R(REG_TEMP.as_index()); // clear upper bits
                    shrq R(REG_TEMP2.as_index()), cl; // a = a >> b
                    movq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // ugly workaround for unsigned shift
                    popq rcx;
                );
            }
            I32Binop::Rotl => {
                monoasm!(
                    &mut self.jit,
                    pushq rcx;
                    movb rcx, R(REG_TEMP2.as_index());
                    andb cl, (0x1F);
                    movl R(REG_TEMP2.as_index()), R(REG_TEMP.as_index()); // clear upper bits
                    roll R(REG_TEMP2.as_index()), cl; // a = a << b
                    movl R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // ugly workaround for rotation
                    popq rcx;
                );
            }
            I32Binop::Rotr => {
                monoasm!(
                    &mut self.jit,
                    pushq rcx;
                    movq rcx, R(REG_TEMP2.as_index());
                    andb cl, (0x1F);
                    movl R(REG_TEMP2.as_index()), R(REG_TEMP.as_index()); // clear upper bits
                    rorl R(REG_TEMP2.as_index()), cl; // a = a >> b
                    movl R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // ugly workaround for rotation
                    popq rcx;
                );
            }
        }

        emit_mov_reg_to_reg(&mut self.jit, a.reg, Register::Reg(REG_TEMP));
        self.reg_allocator.push(a);
    }
}
