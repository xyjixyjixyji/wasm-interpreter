(module
  (global $global_var (mut f64) (f64.const 0))  ;; Define a mutable global variable

  (func (export "main") (param $p f64) (result f64)
    ;; Set the global variable to the value of the parameter
    local.get $p
    global.set $global_var

    ;; Get the value of the global variable and return it
    global.get $global_var
  )
)