use anyhow::Result;
use wasmparser::{Data, Element, Export, FuncType, Global, MemoryType, Table, ValType};

use super::{
    components::{FuncDecl, GlobalDecl, ImportSet},
    insts::Instruction,
    wasm_module::WasmModule,
};

pub(crate) struct FuncBody {
    pub(crate) locals: Vec<(u32, ValType)>,
    pub(crate) insts: Vec<Instruction>,
}

impl<'a> WasmModule<'a> {
    pub(crate) fn parse_type_section(
        tsread: wasmparser::TypeSectionReader,
    ) -> Result<Vec<FuncType>> {
        let mut sigs = vec![];

        for recgroup in tsread {
            let recgroup = recgroup?;
            if recgroup.is_explicit_rec_group() {
                todo!("explicit rec groups not supported");
            } else {
                let ty = recgroup.into_types().next().unwrap();
                match ty.composite_type.inner {
                    wasmparser::CompositeInnerType::Func(func_type) => {
                        sigs.push(func_type);
                    }
                    wasmparser::CompositeInnerType::Array(_)
                    | wasmparser::CompositeInnerType::Struct(_) => {
                        todo!("Array and struct are not yet implemented")
                    }
                }
            }
        }

        Ok(sigs)
    }

    pub(crate) fn parse_import_section(
        iread: wasmparser::ImportSectionReader,
    ) -> Result<ImportSet> {
        let mut import_set = ImportSet {
            imports: vec![],
            num_funcs: 0,
            num_tables: 0,
            num_mems: 0,
            num_globals: 0,
        };

        println!("Import");
        for import in iread {
            println!("Import");
            let import = import?;
            match import.ty {
                wasmparser::TypeRef::Func(_) => import_set.num_funcs += 1,
                wasmparser::TypeRef::Table(_) => import_set.num_tables += 1,
                wasmparser::TypeRef::Memory(_) => import_set.num_mems += 1,
                wasmparser::TypeRef::Global(_) => import_set.num_globals += 1,
                _ => todo!("import tag not yet implemented"),
            }
            import_set.imports.push(import);
        }

        Ok(import_set)
    }

    pub(crate) fn parse_function_section(
        fread: wasmparser::FunctionSectionReader,
        sigs: Vec<FuncType>,
    ) -> Result<Vec<FuncDecl>> {
        let mut func_decls = vec![];

        for ind in fread {
            let ind = ind?;
            let ty = sigs[ind as usize].clone();
            func_decls.push(FuncDecl::new(ty));
        }

        Ok(func_decls)
    }

    pub(crate) fn parse_table_section(
        tread: wasmparser::TableSectionReader<'a>,
    ) -> Result<Vec<Table<'a>>> {
        let mut tables = vec![];

        for table in tread {
            let table = table?;
            tables.push(table);
        }

        Ok(tables)
    }

    pub(crate) fn parse_memory_section(
        memread: wasmparser::MemorySectionReader,
    ) -> Result<Vec<MemoryType>> {
        let mut mems = vec![];

        if memread.count() != 1 {
            anyhow::bail!("multiple memories not yet supported");
        }

        for mem in memread {
            mems.push(mem?);
        }

        Ok(mems)
    }

    pub(crate) fn parse_global_section(
        gread: wasmparser::GlobalSectionReader<'a>,
    ) -> Result<Vec<GlobalDecl>> {
        let mut globals = vec![];
        for global in gread {
            let global = global?;
            let ty = global.ty;
            let mut init_expr = global.init_expr.get_binary_reader();
            let mut init_expr_bytes = vec![];
            while !init_expr.eof() {
                init_expr_bytes.push(init_expr.read_u8()?);
            }

            globals.push(GlobalDecl::new(ty, init_expr_bytes));
        }
        Ok(globals)
    }

    pub(crate) fn parse_export_section(
        eread: wasmparser::ExportSectionReader<'a>,
    ) -> Result<Vec<Export<'a>>> {
        let mut exports = vec![];
        for export in eread {
            exports.push(export?);
        }
        Ok(exports)
    }

    pub(crate) fn parse_element_section(
        eread: wasmparser::ElementSectionReader<'a>,
    ) -> Result<Vec<Element<'a>>> {
        let mut elements = vec![];
        for elem in eread {
            elements.push(elem?);
        }
        Ok(elements)
    }

    pub(crate) fn parse_data_section(
        &self,
        dread: wasmparser::DataSectionReader<'a>,
    ) -> Result<Vec<Data<'a>>> {
        if let Some(count) = self.get_data_count() {
            if dread.count() != count {
                anyhow::bail!("data count section does not match data section size");
            }
        }

        let mut datas = vec![];
        for data in dread {
            datas.push(data?);
        }
        Ok(datas)
    }

    pub(crate) fn parse_code_section(func_body: wasmparser::FunctionBody<'a>) -> Result<FuncBody> {
        let mut locals = vec![];
        let local_reader = func_body.get_locals_reader()?;
        for local in local_reader {
            locals.push(local?);
        }

        let mut binary_reader = func_body.get_binary_reader();
        // skip the locals
        let count = binary_reader.read_var_u32()?;
        for _ in 0..count {
            binary_reader.read_var_u32()?;
            binary_reader.read::<ValType>()?;
        }
        // the remaining bytes are the operators
        let code_bytes = binary_reader
            .read_bytes(binary_reader.bytes_remaining())?
            .to_vec();

        let insts = Instruction::from_code_bytes(code_bytes)?;

        Ok(FuncBody { locals, insts })
    }
}
