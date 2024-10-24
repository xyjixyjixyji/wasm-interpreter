use crate::jit::X86JitCompiler;

impl X86JitCompiler<'_> {
    pub(crate) fn setup_data(&mut self) {
        let module_ref = self.module.borrow();
    }
}