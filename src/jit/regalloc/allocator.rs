// Register allocator for WasmJitCompiler.
// It allocates registers for each instruction based on register vector.
// The compiling context maintains a register vector for each function.
// During the instruction iteration, it updates the register vector accordingly
// based on the Wasm operand stack.

use super::register::{Register, ALLOC_POOL};

pub struct X86RegisterAllocator {
    // Register vector, which is the currently used registers, representing
    // values staying on the wasm operand stack.
    reg_vec: Vec<Register>,
    // Stack offset for the current function frame, used for spilled variables.
    // Note that we always spills 64-bit value.
    stack_offset: usize,
}

impl X86RegisterAllocator {
    pub fn new() -> Self {
        let reg_vec = vec![];
        Self {
            reg_vec,
            stack_offset: 0,
        }
    }

    /// Allocate a position to hold the value.
    pub fn next(&mut self) -> Register {
        let reg = self.next_reg();
        self.reg_vec.push(reg);
        reg
    }

    /// Allocate a position to spill the value. Used for wasm local.
    pub fn new_spill(&mut self) -> Register {
        let reg = self.next_spill();
        self.reg_vec.push(reg);
        reg
    }

    pub fn drop(&mut self) {
        self.reg_vec.pop().expect("no register to drop");
    }
}

impl X86RegisterAllocator {
    // Get the next available register based on current register vector.
    // Iterate through the register pool and return the first available register.
    // If all registers are used, return a stack register.
    fn next_reg(&mut self) -> Register {
        for reg in ALLOC_POOL {
            if !self.reg_vec.contains(&Register::Reg(reg)) {
                return Register::Reg(reg);
            }
        }
        self.next_spill()
    }

    fn next_spill(&mut self) -> Register {
        let reg = Register::Stack(self.stack_offset);
        self.stack_offset += 8;
        reg
    }
}
