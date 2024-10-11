(module
  ;; Import the `puti` function from the host, which takes one i32 argument
  (import "weewasm" "puti" (func $puti (param i32)))

  ;; Start function that calls `puti`
  (func (export "main") (param i32)
    (local.get 0)
    (call $puti)   ;; Call the host function `puti` with 42
  )
)