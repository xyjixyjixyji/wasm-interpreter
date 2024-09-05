(module
  (type $t0 (func (param i32) (result i32)))
  (func $main (export "main") (type $t0) (param $i i32) (result i32)
    local.get $i
    i32.const 1
    i32.sub
    local.set $i
    i32.const 0
    loop $L0 (param i32) (result i32)
      i32.const 10
      i32.add
      i32.const 10
      block $B1 (param i32) (result i32)
        i32.const 100
        i32.add
        i32.const 100
        block $B2 (param i32) (result i32)
          i32.const 1000
          i32.add
          i32.const 1000
          block $B3 (param i32) (result i32)
            i32.const 1
            local.get $i
            i32.const 1
            i32.add
            local.tee $i
            br_table $B2 $B1 $L0 $B3
          end
          i32.add
        end
        i32.add
      end
      i32.add
    end
    return))