use crate::{jit::X86JitCompiler, module::wasmops::WASM_OP_I32_CONST};

impl X86JitCompiler<'_> {
    // table are setup using the element section
    pub(crate) fn setup_tables(&mut self) {
        let module_ref = self.module.borrow();
        let elems = module_ref.get_elems();
        for elem in elems {
            let ind = match &elem.kind {
                wasmparser::ElementKind::Active {
                    table_index,
                    offset_expr,
                } => {
                    if let Some(table_index) = table_index {
                        *table_index
                    } else {
                        let mut reader = offset_expr.get_binary_reader();
                        let op = reader.read_u8().expect(
                            "invalid offset expression when parsing opcode, should be i32.const",
                        );
                        if op as u32 != WASM_OP_I32_CONST {
                            panic!("invalid offset expression when parsing opcode, should be i32.const, op: {}", op);
                        }
                        reader
                            .read_var_i32()
                            .expect("invalid offset expression when parsing value of i32.const")
                            as u32
                    }
                }
                _ => panic!("we dont support passive and declared element segment"),
            };

            // setup the elements in the table
            let items = elem.items.clone();
            match items {
                wasmparser::ElementItems::Functions(r) => {
                    for func_idx in r {
                        self.tables[ind as usize].push(func_idx.unwrap());
                    }
                }
                _ => panic!("we dont support expressions element segment"),
            }
        }
        for (i, table) in self.tables.iter().enumerate() {
            self.table_len[i] = table.len();
        }
    }
}
