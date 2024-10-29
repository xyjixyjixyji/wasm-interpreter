#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86Register {
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

impl X86Register {
    pub fn as_index(&self) -> u64 {
        match self {
            X86Register::Rax => 0,
            X86Register::Rcx => 1,
            X86Register::Rdx => 2,
            X86Register::Rbx => 3,
            X86Register::Rsp => 4,
            X86Register::Rbp => 5,
            X86Register::Rsi => 6,
            X86Register::Rdi => 7,
            X86Register::R8 => 8,
            X86Register::R9 => 9,
            X86Register::R10 => 10,
            X86Register::R11 => 11,
            X86Register::R12 => 12,
            X86Register::R13 => 13,
            X86Register::R14 => 14,
            X86Register::R15 => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86FpRegister {
    Xmm0,
    Xmm1,
    Xmm2,
    Xmm3,
    Xmm4,
    Xmm5,
    Xmm6,
    Xmm7,
    Xmm8,
    Xmm9,
    Xmm10,
    Xmm11,
    Xmm12,
    Xmm13,
    Xmm14,
    Xmm15,
}

impl X86FpRegister {
    pub fn as_index(&self) -> u64 {
        match self {
            X86FpRegister::Xmm0 => 0,
            X86FpRegister::Xmm1 => 1,
            X86FpRegister::Xmm2 => 2,
            X86FpRegister::Xmm3 => 3,
            X86FpRegister::Xmm4 => 4,
            X86FpRegister::Xmm5 => 5,
            X86FpRegister::Xmm6 => 6,
            X86FpRegister::Xmm7 => 7,
            X86FpRegister::Xmm8 => 8,
            X86FpRegister::Xmm9 => 9,
            X86FpRegister::Xmm10 => 10,
            X86FpRegister::Xmm11 => 11,
            X86FpRegister::Xmm12 => 12,
            X86FpRegister::Xmm13 => 13,
            X86FpRegister::Xmm14 => 14,
            X86FpRegister::Xmm15 => 15,
        }
    }
}

pub const REG_LOCAL_BASE: X86Register = X86Register::R12;
pub const REG_TEMP: X86Register = X86Register::R13;
pub const REG_TEMP2: X86Register = X86Register::R14;
pub const REG_MEMORY_BASE: X86Register = X86Register::R15;
pub const REG_TEMP_FP: X86FpRegister = X86FpRegister::Xmm14;
pub const REG_TEMP_FP2: X86FpRegister = X86FpRegister::Xmm15;

pub const ALLOC_POOL: [X86Register; 10] = [
    X86Register::Rax,
    X86Register::Rdi,
    X86Register::Rsi,
    X86Register::Rdx,
    X86Register::Rcx,
    X86Register::R8,
    X86Register::R9,
    X86Register::R10,
    X86Register::Rbx,
    X86Register::R11,
];

pub const FP_ALLOC_POOL: [X86FpRegister; 14] = [
    X86FpRegister::Xmm0,
    X86FpRegister::Xmm1,
    X86FpRegister::Xmm2,
    X86FpRegister::Xmm3,
    X86FpRegister::Xmm4,
    X86FpRegister::Xmm5,
    X86FpRegister::Xmm6,
    X86FpRegister::Xmm7,
    X86FpRegister::Xmm8,
    X86FpRegister::Xmm9,
    X86FpRegister::Xmm10,
    X86FpRegister::Xmm11,
    X86FpRegister::Xmm12,
    X86FpRegister::Xmm13,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    Reg(X86Register),
    FpReg(X86FpRegister),
    Stack(usize), // offset from Rbp
}

impl Register {
    pub fn is_caller_saved(&self) -> bool {
        match self {
            Register::Reg(r) => matches!(
                r,
                X86Register::Rax
                    | X86Register::Rcx
                    | X86Register::Rdx
                    | X86Register::Rsi
                    | X86Register::Rdi
                    | X86Register::R8
                    | X86Register::R9
                    | X86Register::R10
                    | X86Register::R11
            ),
            Register::FpReg(_) => true,
            Register::Stack(_) => false,
        }
    }

    pub fn from_ith_argument(i: u32) -> Register {
        match i {
            0 => Register::Reg(X86Register::Rdi),
            1 => Register::Reg(X86Register::Rsi),
            2 => Register::Reg(X86Register::Rdx),
            3 => Register::Reg(X86Register::Rcx),
            4 => Register::Reg(X86Register::R8),
            5 => Register::Reg(X86Register::R9),
            _ => panic!("invalid argument index: {}", i),
        }
    }
}
impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::Reg(r) => write!(f, "R{}", r.as_index()),
            Register::FpReg(r) => write!(f, "xmm{}", r.as_index()),
            Register::Stack(offset) => write!(f, "[%rbp - {}]", offset),
        }
    }
}
