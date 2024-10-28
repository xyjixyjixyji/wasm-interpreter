use crate::jit::{
    regalloc::{
        Register, X86Register, REG_LOCAL_BASE, REG_MEMORY_BASE, REG_TEMP, REG_TEMP2, REG_TEMP_FP,
    },
    utils::emit_mov_reg_to_reg,
    ValueType, X86JitCompiler,
};

use monoasm::*;
use monoasm_macro::monoasm;

impl X86JitCompiler<'_> {
    pub(crate) fn emit_local_get(
        &mut self,
        dst: Register,
        local_idx: u32,
        local_types: &[ValueType],
    ) {
        let ty = local_types[local_idx as usize];
        let offset = local_idx * 8;
        match ty {
            ValueType::I32 => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), R(REG_LOCAL_BASE.as_index()); // reg_temp = reg_local_base
                    movq R(REG_TEMP.as_index()), [R(REG_TEMP.as_index()) + (offset)];
                );
                emit_mov_reg_to_reg(&mut self.jit, dst, Register::Reg(REG_TEMP));
            }
            ValueType::F64 => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), R(REG_LOCAL_BASE.as_index()); // reg_temp = reg_local_base
                    movq xmm(REG_TEMP_FP.as_index()), [R(REG_TEMP.as_index()) + (offset)];
                );
                emit_mov_reg_to_reg(&mut self.jit, dst, Register::FpReg(REG_TEMP_FP));
            }
        }
    }

    pub(crate) fn emit_local_set(&mut self, value: Register, local_idx: u32, ty: ValueType) {
        let offset = local_idx * 8;
        match ty {
            ValueType::I32 => {
                emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP2), value);
                monoasm!(
                    &mut self.jit,
                    movq [R(REG_LOCAL_BASE.as_index()) + (offset)], R(REG_TEMP2.as_index());
                );
            }
            ValueType::F64 => {
                emit_mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP), value);
                monoasm!(
                    &mut self.jit,
                    movsd [R(REG_LOCAL_BASE.as_index()) + (offset)], xmm(REG_TEMP_FP.as_index());
                );
            }
        }
    }

    pub(crate) fn emit_local_tee(&mut self, top_of_stack: Register, local_idx: u32, ty: ValueType) {
        let offset = local_idx * 8;
        match ty {
            ValueType::I32 => {
                emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP2), top_of_stack);
                monoasm!(
                    &mut self.jit,
                    movq [R(REG_LOCAL_BASE.as_index()) + (offset)], R(REG_TEMP2.as_index());
                );
            }
            ValueType::F64 => {
                emit_mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP), top_of_stack);
                monoasm!(
                    &mut self.jit,
                    movsd [R(REG_LOCAL_BASE.as_index()) + (offset)], xmm(REG_TEMP_FP.as_index());
                );
            }
        }
    }

    pub(crate) fn emit_memory_grow(&mut self, npages: Register) {
        self.linear_mem.grow(&mut self.jit, npages);
    }

    pub(crate) fn emit_load_mem(
        &mut self,
        dst: Register,
        base: Register,
        offset: u32,
        width: u32,
        sign_extend: bool,
    ) {
        self.get_effective_address(REG_TEMP, base, offset); // REG_TEMP stores the effective address

        // 2. load the result into dst
        monoasm!(
            &mut self.jit,
            addq R(REG_TEMP.as_index()), R(REG_MEMORY_BASE.as_index()); // <-- reg_temp = reg_memory_base + effective_addr
        );

        match width {
            8 => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())];
                );
            }
            4 => {
                monoasm!(
                    &mut self.jit,
                    movl R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())];
                );
                if sign_extend {
                    monoasm!(
                        &mut self.jit,
                        movsxl R(REG_TEMP.as_index()), R(REG_TEMP.as_index());
                    );
                }
            }
            2 => {
                monoasm!(
                    &mut self.jit,
                    movw R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())];
                );
                if sign_extend {
                    monoasm!(
                        &mut self.jit,
                        movsxw R(REG_TEMP.as_index()), R(REG_TEMP.as_index());
                    );
                }
            }
            1 => {
                monoasm!(
                    &mut self.jit,
                    movb R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())];
                );
                if sign_extend {
                    monoasm!(
                        &mut self.jit,
                        movsxb R(REG_TEMP.as_index()), R(REG_TEMP.as_index());
                    );
                }
            }
            _ => unreachable!("invalid width: {}", width),
        }

        emit_mov_reg_to_reg(&mut self.jit, dst, Register::Reg(REG_TEMP));
    }

    pub(crate) fn emit_store_mem(
        &mut self,
        base: Register,
        offset: u32,
        value: Register,
        width: u32,
    ) {
        self.get_effective_address(REG_TEMP, base, offset); // reg_temp = effective_addr

        // 2. store the value to dst
        monoasm!(
            &mut self.jit,
            addq R(REG_TEMP.as_index()), R(REG_MEMORY_BASE.as_index()); // <-- reg_temp = reg_memory_base + effective_addr
        );

        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP2), value); // <-- reg_temp = value

        match width {
            8 => {
                monoasm!(
                    &mut self.jit,
                    movq [R(REG_TEMP.as_index())], R(REG_TEMP2.as_index());
                );
            }
            4 => {
                monoasm!(
                    &mut self.jit,
                    movl [R(REG_TEMP.as_index())], R(REG_TEMP2.as_index());
                );
            }
            2 => {
                monoasm!(
                    &mut self.jit,
                    movw [R(REG_TEMP.as_index())], R(REG_TEMP2.as_index());
                );
            }
            1 => {
                monoasm!(
                    &mut self.jit,
                    movb [R(REG_TEMP.as_index())], R(REG_TEMP2.as_index());
                );
            }
            _ => unreachable!("invalid width: {}", width),
        }
    }

    pub(crate) fn emit_global_get(&mut self, dst: Register, global_idx: u32) {
        let global_addr = self.globals.as_ptr() as u64 + (global_idx * 8) as u64;
        monoasm!(
            &mut self.jit,
            movq R(REG_TEMP.as_index()), (global_addr);
            movq R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())];
        );
        emit_mov_reg_to_reg(&mut self.jit, dst, Register::Reg(REG_TEMP));
    }

    pub(crate) fn emit_global_set(&mut self, value: Register, global_idx: u32) {
        let global_addr = self.globals.as_ptr() as u64 + (global_idx * 8) as u64;
        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), value);
        monoasm!(
            &mut self.jit,
            movq R(REG_TEMP2.as_index()), (global_addr);
            movq [R(REG_TEMP2.as_index())], R(REG_TEMP.as_index());
        );
    }

    pub(crate) fn store_mem_page_size(&mut self, dst: Register) {
        self.linear_mem.read_memory_size_in_page(&mut self.jit, dst);
    }

    /// REG_TEMP will store the effective address + width
    fn get_effective_address(&mut self, dst: X86Register, base: Register, offset: u32) {
        emit_mov_reg_to_reg(&mut self.jit, Register::Reg(dst), base); // <-- reg_temp2 = base
        monoasm!(
            &mut self.jit,
            addq R(dst.as_index()), (offset);
        );
    }
}
