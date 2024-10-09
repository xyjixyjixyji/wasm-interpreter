use std::env;

use module::{value_type::WasmValue, wasm_module::WasmModule};

use vm::{WasmInterpreter, WasmVm};

mod module;
mod vm;
mod jit;

struct WasmInterpreterConfig {
    wasm_args: Vec<WasmValue>,
    infile: String,
    jit_mode: bool,
}

fn parse_args() -> WasmInterpreterConfig {
    let args: Vec<String> = env::args().collect();

    let mut wasm_args_str = vec![];
    let mut infile = String::new();
    let mut jit_mode = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--jit" => {
                jit_mode = true;
                i += 1;
            }
            "-a" => {
                i += 1;
                while i < args.len() - 1 {
                    wasm_args_str.push(args[i].clone());
                    i += 1;
                }
            }
            _ => {
                infile = args[i].clone();
                i += 1;
            }
        }
    }

    let wasm_args = wasm_args_str
        .iter()
        .map(|arg| {
            if arg.ends_with("d") {
                let arg = &arg[..arg.len() - 1];
                WasmValue::F64(arg.parse().unwrap())
            } else {
                WasmValue::I32(arg.parse().unwrap())
            }
        })
        .collect();

    WasmInterpreterConfig {
        wasm_args,
        infile,
        jit_mode,
    }
}

fn main() {
    env_logger::init();

    let args = parse_args();

    let wasm_bytes: Vec<u8> = std::fs::read(&args.infile).unwrap();
    let module = WasmModule::from_bytecode(&wasm_bytes);
    let module = match module {
        Ok(module) => module,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    let vm = WasmInterpreter::from_module(module, args.jit_mode);
    match vm.run(args.wasm_args) {
        Ok(r) => {
            print!("{}", r)
        }
        Err(e) => {
            log::debug!("{}", e);
            print!("!trap");
        }
    }
}
