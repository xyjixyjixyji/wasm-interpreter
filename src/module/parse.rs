use anyhow::Result;
use wasmparser::{FuncType, Import};

pub(crate) fn parse_type_section(tsread: wasmparser::TypeSectionReader) -> Result<Vec<FuncType>> {
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

pub(crate) fn parse_import_section(iread: wasmparser::ImportSectionReader) -> Result<Vec<Import>> {
    let mut imports = vec![];

    for import in iread {
        imports.push(import?);
    }

    Ok(imports)
}
