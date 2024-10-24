use crate::{
    jit::regalloc::{REG_MEMORY_BASE, REG_TEMP, REG_TEMP2},
    jit::X86JitCompiler,
    module::wasmops::WASM_OP_I32_CONST,
};

use anyhow::Result;
use monoasm::*;
use monoasm_macro::monoasm;

impl X86JitCompiler<'_> {
    pub(crate) fn setup_data(&mut self) -> Result<()> {
        let module_ref = self.module.borrow();
        for data in module_ref.get_datas() {
            match &data.kind {
                wasmparser::DataKind::Passive => panic!("passive data segment not implemented"),
                wasmparser::DataKind::Active {
                    memory_index,
                    offset_expr,
                } => {
                    if *memory_index != 0 {
                        panic!("data segment memory index should be 0");
                    }

                    let mut reader = offset_expr.get_binary_reader();
                    let op = reader.read_u8()?; // skip WASM_OP_I32_CONST
                    if op as u32 != WASM_OP_I32_CONST {
                        panic!("data segment offset: invalid opcode, should be i32.const");
                    }

                    let offset = usize::try_from(reader.read_var_i32()?)?;
                    let byte_slice = data.data;
                    let byte_slice_ptr = byte_slice.as_ptr();
                    let byte_slice_len = byte_slice.len();

                    // assembly loop to copy bytes to linear memory
                    let loop_label = self.jit.label();
                    let end_label = self.jit.label();
                    monoasm!(
                        &mut self.jit,
                        movq rax, (0); // we are not in the function yet, we can use whatever register
                        movq R(REG_TEMP.as_index()), (byte_slice_ptr);
                    loop_label:
                        cmpq rax, (byte_slice_len);
                        jge end_label;
                        // temp2 = byte_slice[i]
                        movb R(REG_TEMP2.as_index()), [R(REG_TEMP.as_index()) + rax];
                        // memory[offset + i] = byte_slice[i]
                        movb [R(REG_MEMORY_BASE.as_index()) + rax + (offset)], R(REG_TEMP2.as_index());
                        // i++
                        addq rax, (1);
                        jmp loop_label;

                    end_label:
                    );
                }
            }
        }

        Ok(())
    }
}
