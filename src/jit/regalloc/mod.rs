mod allocator;
mod register;

pub use allocator::{RegWithType, X86RegisterAllocator};
pub use register::{
    Register, X86Register, REG_LOCAL_BASE, REG_MEMORY_BASE, REG_TEMP, REG_TEMP2, REG_TEMP_FP,
    REG_TEMP_FP2,
};
