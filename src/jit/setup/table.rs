use crate::jit::X86JitCompiler;

impl X86JitCompiler<'_> {
    // table are setup using the element section
    pub(crate) fn setup_tables(&mut self) {
        let mut n_tables = 0;

        let module_ref = self.module.borrow();
        let elems = module_ref.get_elems();
        for elem in elems {
            let ind: u32;
            match &elem.kind {
                wasmparser::ElementKind::Active { table_index, .. } => {
                    ind = table_index.unwrap();
                    n_tables = n_tables.max(table_index.unwrap() as usize + 1);
                }
                _ => panic!("we dont support passive and declared element segment"),
            }

            // make sure the vector is long enough
            while self.tables.len() < n_tables {
                self.tables.push(Vec::new());
                self.table_len.push(0);
            }

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

            self.table_len[ind as usize] = self.tables[ind as usize].len();
        }
    }
}
