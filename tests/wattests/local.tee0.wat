(module
  (func (export "main") (param $p i32) (result i32)
    (local $local_var i32)
    (local.get $p)
    (local.tee $local_var) ;; store p in local_var, p is still on stack
    (local.get $local_var) ;; we have local_var and p on the stack
    (i32.add) ;; 2 * p
  )
)