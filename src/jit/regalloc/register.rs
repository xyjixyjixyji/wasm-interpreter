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

impl std::fmt::Display for X64Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X64Register::Rax => write!(f, "rax"),
            X64Register::Rbx => write!(f, "rbx"),
            X64Register::Rcx => write!(f, "rcx"),
            X64Register::Rdx => write!(f, "rdx"),
            X64Register::Rsi => write!(f, "rsi"),
            X64Register::Rdi => write!(f, "rdi"),
            X64Register::Rbp => write!(f, "rbp"),
            X64Register::Rsp => write!(f, "rsp"),
            X64Register::R8 => write!(f, "r8"),
            X64Register::R9 => write!(f, "r9"),
            X64Register::R10 => write!(f, "r10"),
            X64Register::R11 => write!(f, "r11"),
            X64Register::R12 => write!(f, "r12"),
            X64Register::R13 => write!(f, "r13"),
            X64Register::R14 => write!(f, "r14"),
            X64Register::R15 => write!(f, "r15"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    Reg(X64Register),
    Stack(usize), // offset from Rsp
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::Reg(r) => write!(f, "{}", r),
            Register::Stack(offset) => write!(f, "[%rsp + {}]", offset),
        }
    }
}
