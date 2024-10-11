(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    block  ;; label = @1
      local.get 0
      i32.const 2
      i32.ge_s
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    local.get 0
    i32.const 2
    i32.sub
    call 0
    local.get 0
    i32.const 1
    i32.sub
    call 0
    i32.add
    return)
  (export "main" (func 0)))
