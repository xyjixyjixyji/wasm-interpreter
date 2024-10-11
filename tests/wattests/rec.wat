(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    global.get 0
    local.get 0
    i32.const 0
    i32.le_s
    br_if 0 (;@0;)
    i32.const 1
    i32.add
    global.set 0
    local.get 0
    i32.const 1
    i32.sub
    call 0)
  (global (;0;) (mut i32) (i32.const 20))
  (export "main" (func 0)))
