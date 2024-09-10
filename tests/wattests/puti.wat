(module
  ;; Import the `puti` function from the host, which takes one i32 argument
  (import "weewasm" "puti" (func $puti (param i32)))

  ;; Start function that calls `puti`
  (func (export "main")
    (i32.const 42) ;; Push the number 42 onto the stack
    (call $puti)   ;; Call the host function `puti` with 42
  )
)