(module
  (type (;0;) (func (result i32)))
  (func (;0;) (type 0) (result i32)
    i32.const 31)
  (func (;1;) (type 0) (result i32)
    i32.const 32)
  (func (;2;) (type 0) (result i32)
    i32.const 33)
  (func (;3;) (type 0) (result i32)
    call 0
    call 1
    i32.add
    call 2
    i32.mul)
  (export "main" (func 3)))
