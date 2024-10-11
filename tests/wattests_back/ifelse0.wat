(module
  (func (export "main") (param $x i32) (param $y i32) (result i32)
    ;; Compare if local 0 (parameter $x) is greater than local 1 (parameter $y)
    (if (result i32)
      (i32.gt_s (local.get $x) (local.get $y)) ;; if $x > $y
      (then
        (i32.const 1) ;; return 1 if true
      )
      (else
        (i32.const 0) ;; return 0 if false
      )
    )
  )
)