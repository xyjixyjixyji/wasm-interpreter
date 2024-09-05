(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (func (;0;) (type 0) (param i32 i32) (result i32)
    i32.const 44
    local.get 0
    br_if 0 (;@0;)
    drop
    i32.const 55
    local.get 1
    br_if 0 (;@0;)
    drop
    i32.const 66)
  (export "main" (func 0)))
