mod allocator;
mod register;

pub use allocator::X86RegisterAllocator;
pub use register::{
    Register, X64Register, REG_LOCAL_BASE, REG_MEMORY_BASE, REG_TEMP, REG_TEMP2, REG_TEMP_FP,
    REG_TEMP_FP2,
};
