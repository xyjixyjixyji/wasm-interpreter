use std::rc::Rc;

use anyhow::Result;
use debug_cell::RefCell;
use monoasm::*;
use monoasm_macro::monoasm;
use regalloc::{Register, REG_TEMP};

use crate::module::{value_type::WasmValue, wasm_module::WasmModule};

pub use compiler::X86JitCompiler;
pub use mem::JitLinearMemory;
pub use trap::register_trap_handler;

pub type ReturnFunc = extern "C" fn() -> u64;

mod compiler;
mod mem;
mod regalloc;
mod trap;

pub trait WasmJitCompiler {
    fn compile(
        &mut self,
        module: Rc<RefCell<WasmModule>>,
        initial_mem_size_in_byte: u64,
        main_params: Vec<WasmValue>,
    ) -> Result<CodePtr>;
}

pub(crate) fn mov_reg_to_reg(jit: &mut JitMemory, dst: Register, src: Register) {
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
