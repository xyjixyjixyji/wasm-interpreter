// Register allocator for WasmJitCompiler.
// It allocates registers for each instruction based on register vector.
// The compiling context maintains a register vector for each function.
// During the instruction iteration, it updates the register vector accordingly
// based on the Wasm operand stack.

use crate::jit::ValueType;

use super::register::{Register, ALLOC_POOL, FP_ALLOC_POOL};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RegWithType {
    pub(crate) reg: Register,
    pub(crate) ty: ValueType,
}

impl RegWithType {
    pub fn new(reg: Register, ty: ValueType) -> Self {
        Self { reg, ty }
    }
}

#[derive(Debug, Clone)]
pub struct X86RegisterAllocator {
    // Register vector, which is the currently used registers, representing
    // values staying on the wasm operand stack.
    reg_vec: Vec<RegWithType>,
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

    pub fn reset(&mut self) {
        self.reg_vec.clear();
        self.stack_offset = 0;
    }

    pub fn clear_vec(&mut self) {
        self.reg_vec.clear();
    }

    pub fn get_vec(&self) -> &Vec<RegWithType> {
        &self.reg_vec
    }

    /// Get the stack top, which is the last element of the register vector.
    pub fn top(&self) -> Option<RegWithType> {
        self.reg_vec.last().copied()
    }

    pub fn size(&self) -> usize {
        self.reg_vec.len()
    }

    pub fn push(&mut self, rt: RegWithType) {
        self.reg_vec.push(rt);
    }

    pub fn pop_noopt(&mut self) -> RegWithType {
        self.reg_vec.pop().expect("no register to drop")
    }

    pub fn pop_opt(&mut self) -> Option<RegWithType> {
        self.reg_vec.pop()
    }

    /// Allocate a position to hold the value.
    pub fn next(&mut self) -> RegWithType {
        let reg = self.next_reg();
        self.reg_vec.push(RegWithType::new(reg, ValueType::I32));
        RegWithType::new(reg, ValueType::I32)
    }

    pub fn next_not_caller_saved(&mut self) -> RegWithType {
        let mut pool: Vec<_> = ALLOC_POOL
            .iter()
            .copied()
            .filter(|r| !Register::Reg(*r).is_caller_saved())
            .filter(|r| !self.reg_vec.iter().any(|rt| rt.reg == Register::Reg(*r)))
            .collect();

        let reg = if pool.is_empty() {
            self.next_spill()
        } else {
            Register::Reg(pool.pop().unwrap())
        };

        self.reg_vec.push(RegWithType::new(reg, ValueType::I32));

        RegWithType::new(reg, ValueType::I32)
    }

    pub fn next_xmm(&mut self) -> RegWithType {
        let reg = self.next_xmm_reg();
        self.reg_vec.push(RegWithType::new(reg, ValueType::F64));
        RegWithType::new(reg, ValueType::F64)
    }

    /// Allocate a position to spill the value. Used for wasm local.
    pub fn new_spill(&mut self, ty: ValueType) -> RegWithType {
        let reg = self.next_spill();
        self.reg_vec.push(RegWithType::new(reg, ty));
        RegWithType::new(reg, ty)
    }

    pub fn get_used_caller_saved_registers(&self) -> Vec<Register> {
        self.reg_vec
            .iter()
            .map(|rt| rt.reg)
            .filter(|r| r.is_caller_saved())
            .collect()
    }
}

impl X86RegisterAllocator {
    // Get the next available register based on current register vector.
    // Iterate through the register pool and return the first available register.
    // If all registers are used, return a stack register.
    fn next_reg(&mut self) -> Register {
        for reg in ALLOC_POOL {
            if !self.reg_vec.iter().any(|rt| rt.reg == Register::Reg(reg)) {
                return Register::Reg(reg);
            }
        }
        self.next_spill()
    }

    fn next_xmm_reg(&mut self) -> Register {
        for reg in FP_ALLOC_POOL {
            if !self.reg_vec.iter().any(|rt| rt.reg == Register::FpReg(reg)) {
                return Register::FpReg(reg);
            }
        }
        self.next_spill()
    }

    fn next_spill(&mut self) -> Register {
        self.stack_offset += 8;
        Register::Stack(self.stack_offset)
    }
}
