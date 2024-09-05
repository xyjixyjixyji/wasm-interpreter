(module
  (memory 1)
  (func (export "main") (param i32 f64) (result f64)
    (f64.store offset=0 (local.get 0) (local.get 1))
    (f64.load offset=0 (local.get 0))
  )
)
