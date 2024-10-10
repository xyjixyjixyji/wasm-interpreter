use super::components::{FuncDecl, GlobalDecl, ImportSet};
use anyhow::Result;
use wasmparser::{Data, Element, Export, FuncType, MemoryType, Parser, Payload::*, Table};

#[derive(Default)]
pub struct WasmModule<'a> {
    sigs: Vec<FuncType>,
    imports: ImportSet<'a>,
    funcs: Vec<FuncDecl>,
    tables: Vec<Table<'a>>,
    mems: Vec<MemoryType>,
    globals: Vec<GlobalDecl>,
    exports: Vec<Export<'a>>,
    elems: Vec<Element<'a>>,
    datas: Vec<Data<'a>>,

    start_func_id: Option<u32>,
    data_count: Option<u32>,
}

impl<'a> WasmModule<'a> {
    pub fn new() -> WasmModule<'a> {
        WasmModule {
            ..Default::default()
        }
    }

    pub fn from_bytecode(bytes: &'a [u8]) -> Result<Self> {
        let parser = Parser::new(0);
        let payloads = parser.parse_all(bytes);

        let mut module = WasmModule::new();

        let mut tot_func: u32 = 0;
        let mut n_func: u32 = 0;

        for payload in payloads {
            match payload? {
                // Sections for WebAssembly modules
                Version { .. } => { /* ... */ }
                TypeSection(tsread) => {
                    module.sigs = Self::parse_type_section(tsread)?;
                }
                ImportSection(iread) => {
                    module.imports = Self::parse_import_section(iread)?;
                    for import in &module.imports.imports {
                        match import.ty {
                            wasmparser::TypeRef::Func(ind) => module
                                .funcs
                                .push(FuncDecl::new(module.sigs[ind as usize].clone())),
                            _ => todo!("import tag not yet implemented"),
                        }
                    }
                }
                FunctionSection(fread) => {
                    if module.funcs.len() != module.get_num_imports() {
                        anyhow::bail!("malformed func imports");
                    }
                    let funcs = Self::parse_function_section(fread, module.sigs.clone())?;
                    module.funcs.extend(funcs);
                }
                TableSection(tread) => {
                    module.tables = Self::parse_table_section(tread)?;
                }
                MemorySection(memread) => {
                    module.mems = Self::parse_memory_section(memread)?;
                }
                GlobalSection(gread) => {
                    module.globals = Self::parse_global_section(gread)?;
                }
                ExportSection(eread) => {
                    module.exports = Self::parse_export_section(eread)?;
                }
                StartSection { func, .. } => module.start_func_id = Some(func),
                ElementSection(eread) => {
                    module.elems = Self::parse_element_section(eread)?;
                }
                DataCountSection { count, .. } => {
                    module.data_count = Some(count);
                }
                DataSection(dread) => {
                    module.datas = module.parse_data_section(dread)?;
                }
                CodeSectionStart { count, .. } => {
                    tot_func = count;
                }
                CodeSectionEntry(body) => {
                    let func_ind = n_func + module.get_num_imports() as u32;
                    let func_ref = module.funcs.get_mut(func_ind as usize).unwrap();
                    func_ref.add_func_body(Self::parse_code_section(body)?);

                    n_func += 1;
                }

                // === The following are not yet implemented ===
                CustomSection(_) => { /* ... */ }

                // most likely you'd return an error here
                UnknownSection { .. } => {
                    panic!("Section id unknown");
                }

                // Sections for WebAssembly components
                TagSection(_) => { /* ... */ }
                ModuleSection { .. } => { /* ... */ }
                InstanceSection(_) => { /* ... */ }
                CoreTypeSection(_) => { /* ... */ }
                ComponentSection { .. } => { /* ... */ }
                ComponentInstanceSection(_) => { /* ... */ }
                ComponentAliasSection(_) => { /* ... */ }
                ComponentTypeSection(_) => { /* ... */ }
                ComponentCanonicalSection(_) => { /* ... */ }
                ComponentStartSection { .. } => { /* ... */ }
                ComponentImportSection(_) => { /* ... */ }
                ComponentExportSection(_) => { /* ... */ }

                // Once we've reached the end of a parser we either resume
                // at the parent parser or the payload iterator is at its
                // end and we're done.
                End(_) => {}
            }
        }

        if n_func != tot_func {
            anyhow::bail!("Function section size mismatch");
        }

        Ok(module)
    }

    pub fn get_sig(&self, index: u32) -> Option<&FuncType> {
        self.sigs.get(index as usize)
    }

    pub fn get_imports(&self) -> &ImportSet<'a> {
        &self.imports
    }

    pub fn get_num_imports(&self) -> usize {
        self.imports.get_num_imports()
    }

    pub fn get_func(&self, index: u32) -> Option<&FuncDecl> {
        self.funcs.get(index as usize)
    }

    pub fn get_funcs(&self) -> &Vec<FuncDecl> {
        &self.funcs
    }

    pub fn get_data_count(&self) -> Option<u32> {
        self.data_count
    }

    pub fn get_datas(&self) -> &Vec<Data<'a>> {
        &self.datas
    }

    pub fn get_memory(&self) -> Option<&MemoryType> {
        self.mems.first()
    }

    pub fn get_exports(&self) -> &Vec<Export<'a>> {
        &self.exports
    }

    pub fn get_globals(&self) -> &Vec<GlobalDecl> {
        &self.globals
    }

    pub fn get_elems(&self) -> &Vec<Element<'a>> {
        &self.elems
    }

    pub fn get_globals_mut(&mut self) -> &mut Vec<GlobalDecl> {
        &mut self.globals
    }
}
