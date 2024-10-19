use monoasm::{Disp, Imm, JitMemory, Reg, Rm, Scale};
use monoasm_macro::monoasm;

use crate::{
    jit::{
        mov_reg_to_reg,
        regalloc::{REG_MEMORY_BASE, REG_TEMP, REG_TEMP2},
    },
    vm::WASM_DEFAULT_PAGE_SIZE_BYTE,
};

use super::regalloc::Register;

pub struct JitLinearMemory {
    size_mem_in_page: Box<u64>,
}

impl JitLinearMemory {
    pub fn new() -> Self {
        Self {
            size_mem_in_page: Box::new(0),
        }
    }

    pub fn init_size(&mut self, jit: &mut JitMemory, initial_mem_size_in_byte: u64) {
        // mmap a 32G region and store in the REG_MEMORY_BASE
        let mem_size_limit: u64 = 32 * 1024 * 1024 * 1024;
        monoasm!(
            &mut *jit,
            xorq rdi, rdi; // addr
            movq rsi, (mem_size_limit); // size
            movq rdx, 0; // PROT_NONE
            movq r10, 0x22; // MAP_PRIVATE | MAP_ANONYMOUS
            movq r8, 0xFFFFFFFFFFFFFFFF; // -1, no fd
            xorq r9, r9; // offset
            movq rax, 9; // mmap
            syscall; // mmap, rax has the pointer to the memory
            movq R(REG_MEMORY_BASE.as_index()), rax;
        );

        let npages = (initial_mem_size_in_byte + WASM_DEFAULT_PAGE_SIZE_BYTE as u64 - 1)
            / WASM_DEFAULT_PAGE_SIZE_BYTE as u64;

        monoasm!(
            &mut *jit,
            movq R(REG_TEMP.as_index()), (npages);
        );

        self.grow(jit, Register::Reg(REG_TEMP));
    }

    pub fn grow(&mut self, jit: &mut JitMemory, npages: Register) {
        log::debug!("memory base: {:x}", self.get_mem_size_addr());
        // get the old size
        self.read_memory_size_in_page(jit, Register::Reg(REG_TEMP)); // reg_temp = old_size
        mov_reg_to_reg(jit, Register::Reg(REG_TEMP2), npages); // reg_temp2 = npages

        // add the old size and npages
        monoasm!(
            &mut *jit,
            addq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // reg_temp = new_size_in_pages
        );

        // store the new size to memory
        monoasm!(
            &mut *jit,
            movq R(REG_TEMP2.as_index()), (self.get_mem_size_addr());
            movq [R(REG_TEMP2.as_index())], R(REG_TEMP.as_index());
        );

        // calculate the new size in bytes
        monoasm!(
            &mut *jit,
            movq R(REG_TEMP2.as_index()), (WASM_DEFAULT_PAGE_SIZE_BYTE as u64);
            imul R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // reg_temp = new_size_in_bytes
        );

        // grow the memory using mprotect
        monoasm!(
            &mut *jit,
            pushq rdi;
            pushq rsi;
            pushq rdx;
            pushq rax;

            movq rdi, R(REG_MEMORY_BASE.as_index()); // rdi = reg_memory_base
            movq rsi, R(REG_TEMP.as_index()); // rsi = new_size_in_bytes
            movq rdx, 0x3; // rdx = PROT_READ | PROT_WRITE
            movq rax, 10; // rax = mprotect
            syscall; // mprotect

            popq rax;
            popq rdx;
            popq rsi;
            popq rdi;
        );
    }

    pub fn read_memory_size_in_page(&self, jit: &mut JitMemory, dst: Register) {
        let mem_size_addr = self.get_mem_size_addr();
        monoasm!(
            &mut *jit,
            movq R(REG_TEMP.as_index()), (mem_size_addr);
            movq R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())];
        );
        mov_reg_to_reg(jit, dst, Register::Reg(REG_TEMP));
    }

    fn get_mem_size_addr(&self) -> u64 {
        Box::<u64>::as_ptr(&self.size_mem_in_page) as u64
    }
}
