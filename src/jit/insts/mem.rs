use crate::jit::{
    mov_reg_to_reg,
    regalloc::{Register, X64Register, REG_LOCAL_BASE, REG_MEMORY_BASE, REG_TEMP, REG_TEMP2},
    ValueType, X86JitCompiler,
};

use monoasm::*;
use monoasm_macro::monoasm;

impl X86JitCompiler {
    pub(crate) fn compile_local_get(
        &mut self,
        dst: Register,
        local_idx: u32,
        local_types: &[ValueType],
    ) {
        let local_type = local_types[local_idx as usize];
        match local_type {
            ValueType::I32 => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), R(REG_LOCAL_BASE.as_index()); // reg_temp = reg_local_base
                    addq R(REG_TEMP.as_index()), (local_idx * 8); // reg_temp = reg_local_base + local_idx * 8
                    movq R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())]; // reg_temp = *reg_temp
                );
                mov_reg_to_reg(&mut self.jit, dst, Register::Reg(REG_TEMP));
            }
            ValueType::F64 => todo!(),
        }
    }

    pub(crate) fn compile_memory_grow(&mut self, npages: Register) {
        self.jit_linear_mem.grow(&mut self.jit, npages);
    }

    pub(crate) fn compile_load(&mut self, dst: Register, base: Register, offset: u32, width: u32) {
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

    pub(crate) fn compile_store(
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
        self.jit_linear_mem
            .read_memory_size_in_page(&mut self.jit, dst);
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
