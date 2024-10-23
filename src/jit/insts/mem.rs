use crate::jit::{
    regalloc::{
        Register, X64Register, REG_LOCAL_BASE, REG_MEMORY_BASE, REG_TEMP, REG_TEMP2, REG_TEMP_FP,
    },
    utils::mov_reg_to_reg,
    ValueType, X86JitCompiler,
};

use monoasm::*;
use monoasm_macro::monoasm;

impl X86JitCompiler {
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
                mov_reg_to_reg(&mut self.jit, dst, Register::Reg(REG_TEMP));
            }
            ValueType::F64 => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), R(REG_LOCAL_BASE.as_index()); // reg_temp = reg_local_base
                    movq xmm(REG_TEMP_FP.as_index()), [R(REG_TEMP.as_index()) + (offset)];
                );
                mov_reg_to_reg(&mut self.jit, dst, Register::FpReg(REG_TEMP_FP));
            }
        }
    }

    pub(crate) fn emit_local_set(
        &mut self,
        value: Register,
        local_idx: u32,
        local_types: &[ValueType],
    ) {
        let ty = local_types[local_idx as usize];
        let offset = local_idx * 8;
        match ty {
            ValueType::I32 => {
                mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP2), value);
                monoasm!(
                    &mut self.jit,
                    movq [R(REG_LOCAL_BASE.as_index()) + (offset)], R(REG_TEMP2.as_index());
                );
            }
            ValueType::F64 => {
                mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP), value);
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

    pub(crate) fn emit_load_mem(&mut self, dst: Register, base: Register, offset: u32, width: u32) {
        self.get_effective_address(REG_TEMP, base, offset); // REG_TEMP stores the effective address

        // 2. load the result into dst
        monoasm!(
            &mut self.jit,
            addq R(REG_TEMP.as_index()), R(REG_MEMORY_BASE.as_index()); // <-- reg_temp = reg_memory_base + effective_addr
            movq R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())]; // <-- reg_temp = *reg_temp
        );

        match width {
            8 => {}
            4 => {
                monoasm!(
                    &mut self.jit,
                    movl R(REG_TEMP.as_index()), R(REG_TEMP.as_index());
                );
            }
            2 => {
                monoasm!(
                    &mut self.jit,
                    movw R(REG_TEMP.as_index()), R(REG_TEMP.as_index());
                );
            }
            1 => {
                monoasm!(
                    &mut self.jit,
                    movb R(REG_TEMP.as_index()), R(REG_TEMP.as_index());
                );
            }
            _ => unreachable!("invalid width: {}", width),
        }

        mov_reg_to_reg(&mut self.jit, dst, Register::Reg(REG_TEMP));
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

        mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP2), value); // <-- reg_temp = value

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

    pub(crate) fn store_mem_page_size(&mut self, dst: Register) {
        self.linear_mem.read_memory_size_in_page(&mut self.jit, dst);
    }

    /// REG_TEMP will store the effective address + width
    fn get_effective_address(&mut self, dst: X64Register, base: Register, offset: u32) {
        mov_reg_to_reg(&mut self.jit, Register::Reg(dst), base); // <-- reg_temp2 = base
        monoasm!(
            &mut self.jit,
            addq R(dst.as_index()), (offset);
        );
    }
}
