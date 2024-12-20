use wasmparser::ValType;

#[derive(Debug, Clone, Copy)]
pub enum WasmValue {
    I32(i32),
    F64(f64),
}

impl std::fmt::Display for WasmValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmValue::I32(val) => write!(f, "{}", val),
            WasmValue::F64(val) => write!(f, "{:.6}", val),
        }
    }
}

impl WasmValue {
    pub fn as_i32(&self) -> i32 {
        match self {
            WasmValue::I32(val) => *val,
            _ => panic!("WasmValue is not I32"),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            WasmValue::F64(val) => *val,
            _ => panic!("WasmValue is not F64"),
        }
    }

    pub fn default_value(value_type: &ValType) -> WasmValue {
        match value_type {
            ValType::I32 => WasmValue::I32(0),
            ValType::F64 => WasmValue::F64(0.0),
            _ => panic!("Unsupported value type"),
        }
    }
}
