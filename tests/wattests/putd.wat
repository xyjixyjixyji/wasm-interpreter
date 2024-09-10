(module
  (import "weewasm" "putd" (func $putd (param f64)))

  (func (export "main") (param f64)
    (local.get 0)
    (call $putd)
  )
)