(module
  ;; Memory setup
  (memory 10)                   ;; Allocate 64KiB of memory
  (export "memory" (memory 0)) ;; Export memory for possible external inspection

  ;; Function to perform matrix multiplication
  (func $matrix_multiply (param $n i32) ;; n is the size of the square matrix
    (local $i i32)      ;; Row index
    (local $j i32)      ;; Column index
    (local $k i32)      ;; Inner loop index
    (local $sum f64)    ;; Accumulator for dot product

    (local.set $i (i32.const 0))
    (loop $outer_i
      (local.set $j (i32.const 0))
      (loop $outer_j
        (local.set $sum (f64.const 0)) ;; Reset sum to 0 for each element in C

        ;; Inner loop for the dot product
        (local.set $k (i32.const 0))
        (loop $inner_k
          ;; Load elements from A and B
          (f64.add
            (local.get $sum)
            (f64.mul
              (f64.load (i32.add (i32.mul (local.get $i) (local.get $n)) (local.get $k)))
              (f64.load (i32.add (i32.mul (local.get $k) (local.get $n)) (local.get $j)))
            )
          )
          local.set $sum

          ;; Increment k and check if we finished the inner loop
          (local.set $k (i32.add (local.get $k) (i32.const 1)))
          (br_if $inner_k (i32.lt_u (local.get $k) (local.get $n)))
        )

        ;; Store the result in matrix C
        (f64.store
          (i32.add (i32.mul (local.get $i) (local.get $n)) (local.get $j))
          (local.get $sum)
        )

        ;; Increment j and check if we finished the outer j loop
        (local.set $j (i32.add (local.get $j) (i32.const 1)))
        (br_if $outer_j (i32.lt_u (local.get $j) (local.get $n)))
      )

      ;; Increment i and check if we finished the outer i loop
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br_if $outer_i (i32.lt_u (local.get $i) (local.get $n)))
    )
  )

  ;; Helper function to initialize matrices A and B with random values
  (func $initialize_matrices (param $n i32)
    (local $i i32)
    (local $j i32)
    (local.set $i (i32.const 0))
    (loop $init_outer
      (local.set $j (i32.const 0))
      (loop $init_inner
        ;; Initialize A[i][j] and B[i][j] with some arbitrary value
        (f64.store (i32.add (i32.mul (local.get $i) (local.get $n)) (local.get $j)) (f64.const 1.1))
        (f64.store (i32.add (i32.mul (local.get $j) (local.get $n)) (local.get $i)) (f64.const 2.2))

        ;; Increment j and check if we finished the inner loop
        (local.set $j (i32.add (local.get $j) (i32.const 1)))
        (br_if $init_inner (i32.lt_u (local.get $j) (local.get $n)))
      )

      ;; Increment i and check if we finished the outer loop
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br_if $init_outer (i32.lt_u (local.get $i) (local.get $n)))
    )
  )

  ;; Main function that initializes matrices and then multiplies them
  (func $main (result i32)
    ;; Call initialize_matrices
    i32.const 300
    call $initialize_matrices
    
    ;; Call matrix_multiply
    i32.const 300
    call $matrix_multiply
    i32.const 1
  )
  (export "main" (func $main))
)