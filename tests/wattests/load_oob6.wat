(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    local.get 0
    f64.load offset=60000
    drop
    i32.const 143)
  (memory (;0;) 1)
  (export "main" (func 0)))
