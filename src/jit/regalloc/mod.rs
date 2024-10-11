mod allocator;
mod register;

pub use allocator::X86RegisterAllocator;
pub use register::{Register, X64Register, REG_MEMORY_BASE, REG_TEMP};
