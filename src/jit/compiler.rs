use super::regalloc::{Register, X64Register, X86RegisterAllocator};
use super::WasmJitCompiler;
use crate::module::components::FuncDecl;
use crate::module::insts::Instruction;
use crate::module::wasm_module::WasmModule;

use anyhow::Result;
use monoasm::{CodePtr, Disp, Imm, JitMemory, Reg, Rm, Scale};
use monoasm_macro::monoasm;

// Jit compile through abstract interpretation
pub struct X86JitCompiler {
    reg_allocator: X86RegisterAllocator,
    jit: JitMemory,
}

impl X86JitCompiler {
    pub fn new() -> Self {
        Self {
            reg_allocator: X86RegisterAllocator::new(),
            jit: JitMemory::new(),
        }
    }
}

impl WasmJitCompiler for X86JitCompiler {
    fn compile(&mut self, fdecl: &FuncDecl) -> Result<CodePtr> {
        let mut codeptr = self.jit.get_current_address();
        for inst in fdecl.get_insts() {
            match inst {
                Instruction::Unreachable => todo!(),
                Instruction::Nop => {}
                Instruction::Block { ty } => todo!(),
                Instruction::Loop { ty } => todo!(),
                Instruction::If { ty } => todo!(),
                Instruction::Else => todo!(),
                Instruction::End => {}
                Instruction::Br { rel_depth } => todo!(),
                Instruction::BrIf { rel_depth } => todo!(),
                Instruction::BrTable { table } => todo!(),
                Instruction::Return => todo!(),
                // For function calls, before we jit compile any program
                Instruction::Call { func_idx } => todo!(),
                Instruction::CallIndirect {
                    type_index,
                    table_index,
                } => todo!(),
                Instruction::Drop => todo!(),
                Instruction::Select => todo!(),
                Instruction::LocalGet { local_idx } => todo!(),
                Instruction::LocalSet { local_idx } => todo!(),
                Instruction::LocalTee { local_idx } => todo!(),
                Instruction::GlobalGet { global_idx } => todo!(),
                Instruction::GlobalSet { global_idx } => todo!(),
                Instruction::I32Load { memarg } => todo!(),
                Instruction::F64Load { memarg } => todo!(),
                Instruction::I32Load8S { memarg } => todo!(),
                Instruction::I32Load8U { memarg } => todo!(),
                Instruction::I32Load16S { memarg } => todo!(),
                Instruction::I32Load16U { memarg } => todo!(),
                Instruction::I32Store { memarg } => todo!(),
                Instruction::F64Store { memarg } => todo!(),
                Instruction::I32Store8 { memarg } => todo!(),
                Instruction::I32Store16 { memarg } => todo!(),
                Instruction::MemorySize { mem } => todo!(),
                Instruction::MemoryGrow { mem } => todo!(),
                Instruction::I32Const { value } => {
                    let reg = self.reg_allocator.next();
                    self.mov_i32_to_reg(*value, reg);
                }
                Instruction::F64Const { value } => todo!(),
                Instruction::I32Unop(_) => todo!(),
                Instruction::I32Binp(_) => todo!(),
                Instruction::F64Unop(_) => todo!(),
                Instruction::F64Binop(_) => todo!(),
            }
        }

        // return...
        monoasm!(
            &mut self.jit,
            ret;
        );

        Ok(codeptr)
    }
}

impl X86JitCompiler {
    fn mov_i32_to_reg(&mut self, value: i32, reg: Register) {
        match reg {
            Register::Reg(r) => match r {
                X64Register::Rax => monoasm!(
                    &mut self.jit,
                    movq rax, (value);
                ),
                X64Register::Rbx => monoasm!(
                    &mut self.jit,
                    movq rbx, (value);
                ),
                X64Register::Rcx => monoasm!(
                    &mut self.jit,
                    movq rcx, (value);
                ),
                X64Register::Rdx => monoasm!(
                    &mut self.jit,
                    movq rdx, (value);
                ),
                X64Register::Rsi => monoasm!(
                    &mut self.jit,
                    movq rsi, (value);
                ),
                X64Register::Rdi => monoasm!(
                    &mut self.jit,
                    movq rdi, (value);
                ),
                X64Register::Rbp => monoasm!(
                    &mut self.jit,
                    movq rbp, (value);
                ),
                X64Register::Rsp => monoasm!(
                    &mut self.jit,
                    movq rsp, (value);
                ),
                X64Register::R8 => monoasm!(
                    &mut self.jit,
                    movq r8, (value);
                ),
                X64Register::R9 => monoasm!(
                    &mut self.jit,
                    movq r9, (value);
                ),
                X64Register::R10 => monoasm!(
                    &mut self.jit,
                    movq r10, (value);
                ),
                X64Register::R11 => monoasm!(
                    &mut self.jit,
                    movq r11, (value);
                ),
                X64Register::R12 => monoasm!(
                    &mut self.jit,
                    movq r12, (value);
                ),
                X64Register::R13 => monoasm!(
                    &mut self.jit,
                    movq r13, (value);
                ),
                X64Register::R14 => monoasm!(
                    &mut self.jit,
                    movq r14, (value);
                ),
                X64Register::R15 => monoasm!(
                    &mut self.jit,
                    movq r15, (value);
                ),
            },
            Register::Stack(offset) => {
                monoasm!(
                    &mut self.jit,
                    movq [rsp + (offset)], (value);
                );
            }
        }
    }
}
