pub struct wasm_limits {
    pub initial: u32,
    pub max: u32,
    pub has_max: bool,
    pub flag: bool,
}

pub struct wasm_localcse_t {
    pub count: u32,
}
