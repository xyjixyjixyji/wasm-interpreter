use super::{
    regalloc::{Register, REG_TEMP},
    X86JitCompiler,
};

use monoasm::*;
use monoasm_macro::monoasm;

/// This uses REG_TEMP as a temporary register only.
pub(crate) fn emit_mov_reg_to_reg(jit: &mut JitMemory, dst: Register, src: Register) {
    if dst == src {
        return;
    }

    match (dst, src) {
        (Register::Stack(o_dst), Register::Stack(o_src)) => {
            monoasm!(
                &mut *jit,
                movq R(REG_TEMP.as_index()), [rsp + (o_src)];
                movq [rsp + (o_dst)], R(REG_TEMP.as_index());
            );
        }
        (Register::Reg(r_dst), Register::Stack(o_src)) => {
            monoasm!(
                &mut *jit,
                movq R(r_dst.as_index()), [rsp + (o_src)];
            );
        }
        (Register::FpReg(fpr_dst), Register::Stack(o_src)) => {
            monoasm!(
                &mut *jit,
                movq xmm(fpr_dst.as_index()), [rsp + (o_src)];
            );
        }
        (Register::Reg(r_dst), Register::Reg(r_src)) => {
            monoasm!(
                &mut *jit,
                movq R(r_dst.as_index()), R(r_src.as_index());
            );
        }
        (Register::Reg(r_dst), Register::FpReg(fpr_src)) => {
            monoasm!(
                &mut *jit,
                movq R(r_dst.as_index()), xmm(fpr_src.as_index());
            );
        }
        (Register::FpReg(fpr_dst), Register::Reg(r_src)) => {
            monoasm!(
                &mut *jit,
                movq xmm(fpr_dst.as_index()), R(r_src.as_index());
            );
        }
        (Register::FpReg(fpr_dst), Register::FpReg(fpr_src)) => {
            monoasm!(
                &mut *jit,
                movq xmm(fpr_dst.as_index()), xmm(fpr_src.as_index());
            );
        }
        (Register::Stack(o_dst), Register::Reg(r_src)) => {
            monoasm!(
                &mut *jit,
                movq [rsp + (o_dst)], R(r_src.as_index());
            );
        }
        (Register::Stack(o_dst), Register::FpReg(fpr_src)) => {
            monoasm!(
                &mut *jit,
                movq [rsp + (o_dst)], xmm(fpr_src.as_index());
            );
        }
    }
}

impl X86JitCompiler<'_> {
    pub(crate) fn emit_mov_i32_to_reg(&mut self, value: i32, reg: Register) {
        match reg {
            Register::Reg(r) => {
                monoasm!(
                    &mut self.jit,
                    movq R(r.as_index()), (value);
                );
            }
            Register::Stack(offset) => {
                monoasm!(
                    &mut self.jit,
                    movq [rsp + (offset)], (value);
                );
            }
            Register::FpReg(_) => panic!("invalid mov for i32 to fp reg"),
        }
    }

    pub(crate) fn emit_mov_f64_to_reg(&mut self, value: f64, reg: Register) {
        let bits = value.to_bits();
        match reg {
            Register::FpReg(r) => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), (bits);
                    movq xmm(r.as_index()), R(REG_TEMP.as_index());
                );
            }
            Register::Stack(offset) => {
                monoasm!(
                    &mut self.jit,
                    movq [rsp + (offset)], (bits);
                );
            }
            _ => panic!("invalid mov for f32 to normal reg"),
        }
    }
}
