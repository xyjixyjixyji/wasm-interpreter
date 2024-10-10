#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X64Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    Rbp,
    Rsp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15, // Reserved for linear memory based address
}

impl X64Register {
    pub fn as_index(&self) -> u64 {
        match self {
            X64Register::Rax => 0,
            X64Register::Rcx => 1,
            X64Register::Rdx => 2,
            X64Register::Rbx => 3,
            X64Register::Rsp => 4,
            X64Register::Rbp => 5,
            X64Register::Rsi => 6,
            X64Register::Rdi => 7,
            X64Register::R8 => 8,
            X64Register::R9 => 9,
            X64Register::R10 => 10,
            X64Register::R11 => 11,
            X64Register::R12 => 12,
            X64Register::R13 => 13,
            X64Register::R14 => 14,
            X64Register::R15 => 15,
        }
    }
}

pub const REG_MEMORY_BASE: X64Register = X64Register::R15;

pub const ALLOC_POOL: [X64Register; 13] = [
    X64Register::Rax,
    X64Register::Rdi,
    X64Register::Rsi,
    X64Register::Rdx,
    X64Register::Rcx,
    X64Register::R8,
    X64Register::R9,
    X64Register::R10,
    X64Register::Rbx,
    X64Register::R11,
    X64Register::R12,
    X64Register::R13,
    X64Register::R14,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    Reg(X64Register),
    Stack(usize), // offset from Rsp
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::Reg(r) => write!(f, "R{}", r.as_index()),
            Register::Stack(offset) => write!(f, "[%rsp + {}]", offset),
        }
    }
}
