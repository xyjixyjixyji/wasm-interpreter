(module
  (type (;0;) (func (param i32) (result i32)))
  (func $main_stub (type 0) (param i32) (result i32)
    (local i32 i32 i32)
    i32.const 1
    local.set 1
    local.get 0
    local.set 0
    loop  ;; label = @1
      local.get 1
      local.tee 2
      i32.const 1
      i32.add
      local.tee 3
      local.set 1
      local.get 0
      local.get 2
      i32.div_s
      i32.const -13
      i32.add
      local.tee 2
      local.set 0
      local.get 3
      i32.const 7
      i32.ne
      br_if 0 (;@1;)
    end
    local.get 2)
  (table (;0;) 1 1 funcref)
  (memory (;0;) 2)
  (global $__stack_pointer (mut i32) (i32.const 66560))
  (export "memory" (memory 0))
  (export "main" (func $main_stub)))
