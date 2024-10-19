use anyhow::{anyhow, Result};
use debug_cell::RefCell;

use std::rc::Rc;

use crate::{
    jit::{register_trap_handler, ReturnFunc, WasmJitCompiler, X86JitCompiler},
    module::{
        components::FuncDecl, value_type::WasmValue, wasm_module::WasmModule,
        wasmops::WASM_OP_I32_CONST,
    },
    vm::WASM_DEFAULT_PAGE_SIZE_BYTE,
};

use super::{func_exec::WasmFunctionExecutorImpl, WasmFunctionExecutor, WasmVm};

pub(crate) struct LinearMemory(pub(crate) Vec<u8>);

impl LinearMemory {
    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn grow(&mut self, additional_pages: u32) {
        let new_size = self.0.len() + (additional_pages as usize * WASM_DEFAULT_PAGE_SIZE_BYTE);
        self.0.resize(new_size, 0);
    }
}

pub struct WasmInterpreter<'a> {
    module: Rc<RefCell<WasmModule<'a>>>,
    mem: Rc<RefCell<LinearMemory>>,
    jit_mode: bool,
}

impl WasmVm for WasmInterpreter<'_> {
    fn run(&self, main_params: Vec<WasmValue>) -> anyhow::Result<String> {
        // find main from export to run
        let main_func = {
            let module_ref = self.module.borrow();
            let main_index = module_ref
                .get_main_index()
                .expect("main function not found");
            module_ref
                .get_func(main_index)
                .ok_or_else(|| anyhow!("main function not found"))?
                .clone()
        };

        let result = if self.jit_mode {
            log::debug!("Running in JIT mode");
            self.run_jit(main_func, main_params)?
        } else {
            log::debug!("Running in interpreter mode");
            self.run_interpreter(main_func, main_params)?
        };

        Ok(result)
    }
}

impl WasmInterpreter<'_> {
    fn run_jit(&self, main_func: FuncDecl, main_params: Vec<WasmValue>) -> Result<String> {
        // register trap handler for SIGSEGV, which is used when wasm code has
        // error. There, we print "!trap" and exit.
        register_trap_handler();

        // jit compile all functions
        // vm_entry is an opaque entry point to the typed main function
        let mut compiler = X86JitCompiler::new();
        let vm_entry = compiler.compile(
            Rc::clone(&self.module),
            self.mem.borrow().0.len() as u64,
            main_params,
        )?;

        // invoke main
        let result = match main_func.get_sig().results()[0] {
            wasmparser::ValType::I32 => {
                let f: ReturnFunc = unsafe { std::mem::transmute(vm_entry) };
                // If you want to step over......
                unsafe {
                    std::intrinsics::breakpoint();
                }
                WasmValue::I32(f() as i32).to_string()
            }
            wasmparser::ValType::F64 => {
                let f: ReturnFunc = unsafe { std::mem::transmute(vm_entry) };
                WasmValue::F64(f64::from_bits(f())).to_string()
            }
            _ => unimplemented!(),
        };

        Ok(result)
    }

    fn run_interpreter(&self, main_func: FuncDecl, main_params: Vec<WasmValue>) -> Result<String> {
        let mut executor = WasmFunctionExecutorImpl::new(
            main_func,
            Rc::clone(&self.module),
            Rc::clone(&self.mem),
            Some(main_params),
        );

        let result = executor.execute()?;
        let result = match result {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        Ok(result)
    }
}

impl<'a> WasmInterpreter<'a> {
    pub fn from_module(module: WasmModule<'a>, jit_mode: bool) -> Self {
        let mut mem = LinearMemory(if let Some(mem) = module.get_memory() {
            vec![0; mem.initial as usize * WASM_DEFAULT_PAGE_SIZE_BYTE]
        } else {
            vec![]
        });

        Self::setup_data_section(&module, &mut mem).expect("failed to setup data section");

        WasmInterpreter {
            module: Rc::new(RefCell::new(module)),
            mem: Rc::new(RefCell::new(mem)),
            jit_mode,
        }
    }
}

impl<'a> WasmInterpreter<'a> {
    /// setup data section with the given data section in the module
    /// e.g. (data (i32.const 10) "foo") will be loaded to linear memory at address 10
    fn setup_data_section(module: &WasmModule<'a>, mem: &mut LinearMemory) -> Result<()> {
        let datas = module.get_datas();
        for data in datas {
            match &data.kind {
                wasmparser::DataKind::Passive => panic!("passive data segment not implemented"),
                wasmparser::DataKind::Active {
                    memory_index,
                    offset_expr,
                } => {
                    if *memory_index != 0 {
                        return Err(anyhow!("memory.init: invalid memory index"));
                    }

                    // read offset_index
                    let mut reader = offset_expr.get_binary_reader();
                    let op = reader.read_u8()?; // skip WASM_OP_I32_CONST
                    if op as u32 != WASM_OP_I32_CONST {
                        panic!("data segment offset: invalid opcode, should be i32.const");
                    }

                    let offset = reader.read_var_i32()?;
                    let byte_slice = data.data;

                    let offset = usize::try_from(offset)?;
                    for (i, b) in byte_slice.iter().enumerate() {
                        mem.0[offset + i] = *b;
                    }
                }
            }
        }

        Ok(())
    }
}
