(module
  (type (;0;) (func (result i32)))
  (type (;1;) (func (param i32) (result i32)))
  (func (;0;) (type 1) (param i32) (result i32)
    local.get 0
    call_indirect (type 0))
  (table (;0;) 1 1 funcref)
  (export "main" (func 0))
  (elem (;0;) (i32.const 0) func 0))
