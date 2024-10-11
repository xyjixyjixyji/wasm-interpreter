//! Trap handler for wasm jit program. We trigger trap when wasm code has
//! invalid instructions. For example, when we divide by zero, or when we
//! access invalid memory address, or we reach unreachable instruction.
//!
//! The way we do this is to trigger sigsegv whenever trap happens, and here
//! we print "!trap" and exit.

use libc::{sigaction, siginfo_t, SIGSEGV};

extern "C" fn trap_handler(signum: i32, _info: *mut siginfo_t, _ctx: *mut libc::c_void) {
    if signum == SIGSEGV {
        print!("!trap");
        std::process::exit(0);
    }
}

pub fn register_trap_handler() {
    unsafe {
        let mut sa: sigaction = std::mem::zeroed();
        sa.sa_sigaction = trap_handler as usize;
        sa.sa_flags = libc::SA_SIGINFO;
        sigaction(SIGSEGV, &sa, std::ptr::null_mut());
    }
}
