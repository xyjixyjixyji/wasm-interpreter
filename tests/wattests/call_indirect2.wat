(module
  (type (;0;) (func (result i32)))
  (type (;1;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (result i32)
    i32.const 11)
  (func (;1;) (type 0) (result i32)
    i32.const 22)
  (func (;2;) (type 0) (result i32)
    i32.const 33)
  (func (;3;) (type 0) (result i32)
    i32.const 44)
  (func (;4;) (type 1) (param i32) (result i32)
    local.get 0
    call_indirect (type 0))
  (table (;0;) 5 5 funcref)
  (export "main" (func 4))
  (elem (;0;) (i32.const 0) func 0 1 2 3 4))
