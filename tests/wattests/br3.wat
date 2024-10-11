(module
  (type (;0;) (func (result i32)))
  (func (;0;) (type 0) (result i32)
    block  ;; label = @1
      block  ;; label = @2
        i32.const 53
        br 2 (;@0;)
        drop
      end
    end
    i32.const 67)
  (export "main" (func 0)))
