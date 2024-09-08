(module
  (func $k (param i32 i32) (result i32) 
    (i32.sub (local.get 0) (local.get 1))
  )
  (func (export "main") (result i32)
    (call $k (i32.const 1) (i32.const 2))
  )
)
