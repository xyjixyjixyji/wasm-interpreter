(module
  (type (;0;) (func (result f64)))
  (func (;0;) (type 0) (result f64)
    i32.const 1000
    f64.const 0x1.31911001c4951p+1 (;=2.38724;)
    f64.store
    i32.const 1000
    f64.load)
  (memory (;0;) 1)
  (export "main" (func 0)))
