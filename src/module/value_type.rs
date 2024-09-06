pub enum WasmValue {
    I32(i32),
    F64(f64),
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
}
