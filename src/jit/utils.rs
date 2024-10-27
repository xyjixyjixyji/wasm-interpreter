use crate::module::{components::FuncDecl, insts::Instruction};

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
    pub(crate) fn emit_mov_rawvalue_to_reg(&mut self, value: u64, reg: Register) {
        match reg {
            Register::Reg(r) => {
                monoasm!(
                    &mut self.jit,
                    movq R(r.as_index()), (value);
                );
            }
            Register::FpReg(r) => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), (value);
                    movq xmm(r.as_index()), R(REG_TEMP.as_index());
                );
            }
            Register::Stack(offset) => {
                monoasm!(
                    &mut self.jit,
                    movq [rsp + (offset)], (value);
                );
            }
        }
    }

    // Get the stack size usage of the function, used for stack allocation
    // We get only an upper bound approximate, since we don't want too much overhead
    pub(crate) fn get_stack_size_in_byte(&self, fdecl: &FuncDecl) -> u64 {
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
