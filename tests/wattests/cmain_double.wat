(module
  (type (;0;) (func (param f64 f64) (result f64)))
  (func $main_stub (type 0) (param f64 f64) (result f64)
    (local i32 i32 f64)
    i32.const 8
    local.set 2
    local.get 0
    local.set 0
    loop  ;; label = @1
      local.get 2
      i32.const -1
      i32.add
      local.tee 3
      local.set 2
      local.get 0
      f64.const 0x1.0cccccccccccdp+1 (;=2.1;)
      f64.mul
      local.get 1
      f64.add
      local.tee 4
      local.set 0
      local.get 3
      br_if 0 (;@1;)
    end
    local.get 4)
  (table (;0;) 1 1 funcref)
  (memory (;0;) 2)
  (global $__stack_pointer (mut i32) (i32.const 66560))
  (export "memory" (memory 0))
  (export "main" (func $main_stub)))
