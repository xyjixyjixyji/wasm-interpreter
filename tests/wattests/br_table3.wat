(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        br_table 1 0
        i32.const 21
        return
      end
      i32.const 20
      return
    end
    i32.const 22)
  (export "main" (func 0)))
