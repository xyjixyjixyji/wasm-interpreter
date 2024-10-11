(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    i32.const 1000
    i32.const 573785173
    i32.store8
    local.get 0
    i32.load)
  (memory (;0;) 1)
  (export "main" (func 0)))
