use anyhow::Result;
use wasmparser::{FuncType, Import};

use super::{ImportSet, WasmModule};

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

        for import in iread {
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
        &self,
        fread: wasmparser::FunctionSectionReader,
    ) -> Result<Vec<FuncType>> {
        let mut funcs = vec![];

        if fread.count() != self.get_num_imports() as u32 {
            anyhow::bail!(
                "malformed func imports, function section size does not match import section size"
            );
        }

        for ind in fread {
            let ind = ind?;
            let ty = self.sigs[ind as usize].clone();
            funcs.push(ty);
        }

        Ok(funcs)
    }
}
