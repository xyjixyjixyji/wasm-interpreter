use monoasm::{Disp, Imm, JitMemory, Reg, Rm, Scale};
use monoasm_macro::monoasm;

use crate::jit::regalloc::{REG_MEMORY_BASE, REG_TEMP, REG_TEMP2};

use super::regalloc::Register;

pub struct JitLinearMemory {
    mem: Vec<u8>,
    size_mem: Box<u64>,
}

impl JitLinearMemory {
    pub fn new() -> Self {
        Self {
            mem: vec![],
            size_mem: Box::new(0),
        }
    }

    pub fn save_base(&mut self, jit: &mut JitMemory, initial_mem_size_in_byte: u64) {
        self.mem = vec![0u8; initial_mem_size_in_byte as usize];
        let base = self.mem.as_ptr() as u64;
        monoasm!(
            jit,
            movq R(REG_MEMORY_BASE.as_index()), (base);
        )
    }

    pub fn save_size_in_pages(&mut self, jit: &mut JitMemory, size: u64) {
        let size_mem_addr = Box::<u64>::as_ptr(&self.size_mem);
        monoasm!(
            jit,
            movq R(REG_TEMP.as_index()), (size);
            movq R(REG_TEMP2.as_index()), (size_mem_addr);
            movq [R(REG_TEMP2.as_index())], R(REG_TEMP.as_index());
        )
    }

    pub fn read(&self, dst: Register, addr: u64, size: u32) {
        todo!()
    }

    pub fn grow(&mut self, dst: Register, jit: &mut JitMemory, additional_pages: u64) {
        todo!()
    }
}
