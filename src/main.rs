use std::env;

use module::WasmModule;

mod module;

#[derive(Debug)]
struct WasmInterpreterArgs {
    wasm_args: Vec<String>,
    infile: String,
}

fn parse_args() -> WasmInterpreterArgs {
    let args: Vec<String> = env::args().collect();

    let mut wasm_args = vec![];
    let mut infile = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-a" => {
                i += 1;
                while i < args.len() - 1 {
                    wasm_args.push(args[i].clone());
                    i += 1;
                }
            }
            _ => {
                infile = args[i].clone();
                i += 1;
            }
        }
    }

    WasmInterpreterArgs { wasm_args, infile }
}

fn main() {
    let args = parse_args();

    let wasm_bytes: Vec<u8> = std::fs::read(&args.infile).unwrap();
    let module = WasmModule::from_bytecode(&wasm_bytes).unwrap();

    println!("{:?}", args);
}
