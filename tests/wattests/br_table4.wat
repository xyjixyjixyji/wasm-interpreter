(module
  (type (;0;) (func (param i32) (result i32)))
  (func (;0;) (type 0) (param i32) (result i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                local.get 0
                br_table 0 (;@6;) 1 (;@5;) 2 (;@4;) 3 (;@3;) 4 (;@2;) 5 (;@1;) 5 (;@1;) 4 (;@2;) 3 (;@3;) 2 (;@4;) 1 (;@5;) 0 (;@6;) 0 (;@6;) 0 (;@6;) 1 (;@5;) 1 (;@5;) 2 (;@4;) 2 (;@4;)
                i32.const 44
                return
              end
              i32.const 45
              return
            end
            i32.const 46
            return
          end
          i32.const 47
          return
        end
        i32.const 48
        return
      end
      i32.const 49
      return
    end
    i32.const 56)
  (export "main" (func 0)))
