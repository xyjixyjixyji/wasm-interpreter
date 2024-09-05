pub mod parse;
pub mod wasmdefs;
pub mod wasmops;

use self::parse::*;

use anyhow::Result;
use wasmparser::{Chunk, FuncType, Import, Parser, Payload::*};

#[derive(Default)]
pub struct WasmModule<'a> {
    sigs: Vec<FuncType>,
    imports: Vec<Import<'a>>,
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

        for payload in payloads {
            match payload? {
                // Sections for WebAssembly modules
                Version { .. } => { /* ... */ }
                TypeSection(tsread) => {
                    module = module.sigs(parse_type_section(tsread)?);
                }
                ImportSection(iread) => {
                    module = module.imports(parse_import_section(iread)?);
                }
                FunctionSection(fread) => { /* ... */ }
                TableSection(tbread) => { /* ... */ }
                MemorySection(memread) => { /* ... */ }
                GlobalSection(gread) => { /* ... */ }
                ExportSection(eread) => { /* ... */ }
                StartSection { func, range } => { /* ... */ }
                ElementSection(eread) => { /* ... */ }
                DataCountSection { count, range } => { /* ... */ }
                DataSection(dread) => { /* ... */ }

                // Here we know how many functions we'll be receiving as
                // `CodeSectionEntry`, so we can prepare for that, and
                // afterwards we can parse and handle each function
                // individually.
                CodeSectionStart { count, range, size } => { /* ... */ }
                CodeSectionEntry(body) => {
                    // here we can iterate over `body` to parse the function
                    // and its locals
                }

                CustomSection(cread) => { /* ... */ }

                // most likely you'd return an error here
                UnknownSection { .. } => { /* ... */ }

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
        Ok(module)
    }

    pub fn sigs(mut self, sigs: Vec<FuncType>) -> Self {
        self.sigs = sigs;
        self
    }

    pub fn imports(mut self, imports: Vec<Import<'a>>) -> Self {
        self.imports = imports;
        self
    }
}
