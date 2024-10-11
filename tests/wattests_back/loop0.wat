(module
  ;; Define the memory and export it
  (memory 1)
  (export "memory" (memory 0))

  ;; Function to increment the local variable
  (func (export "main") (result i32)
    ;; Local variable to store the incremented value
    (local $i i32)   ;; This is the variable to increment
    (local $counter i32)  ;; Loop counter

    ;; Initialize the local variables
    (local.set $i (i32.const 0))       ;; $i starts at 0
    (local.set $counter (i32.const 20))  ;; $counter is set to the input

    ;; Loop label
    (block $exit
      (loop $loop
        ;; Break out of the loop if the counter reaches 0
        (local.get $counter)
        (i32.eqz)
        (br_if $exit)

        ;; Increment the local variable $i
        (local.get $i)
        (i32.const 1)
        (i32.add)
        (local.set $i)

        ;; Decrement the counter
        (local.get $counter)
        (i32.const 1)
        (i32.sub)
        (local.set $counter)

        ;; Repeat the loop
        (br $loop)
      )
    )

    ;; Return the incremented local $i
    (local.get $i)
  )
)
