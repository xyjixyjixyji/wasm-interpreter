(module
  (type (;0;) (func (param i32 i32 i32) (result i32)))
  (func (;0;) (type 0) (param i32 i32 i32) (result i32)
    block  ;; label = @1
      i32.const 11
      local.get 0
      br_if 1 (;@0;)
      drop
      i32.const 22
      local.get 1
      br_if 1 (;@0;)
      drop
      i32.const 33
      local.get 2
      br_if 1 (;@0;)
      drop
    end
    i32.const 44)
  (export "main" (func 0)))
