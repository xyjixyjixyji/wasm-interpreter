(module
  (func (export "main") (result i32)
    (f64.eq (f64.abs (f64.const -0x1.0000012345678p12)) (f64.const 0x1.0000012345678p12))
    (f64.eq (f64.abs (f64.const 0x1.5555598765432p342)) (f64.const 0x1.5555598765432p342))
    (f64.eq (f64.abs (f64.const 0)) (f64.const 0))

    (f64.eq (f64.add (f64.const 0x1.A000000000000p2)(f64.const 0x1.6800000000000p3)) (f64.const 0x1.1C00000000000p4))
    (f64.eq (f64.add (f64.const 0) (f64.const 0)) (f64.const 0))

    (f64.eq (f64.ceil (f64.const 0x1.8000000000000p0)) (f64.const 0x1.0000000000000p1))
    (f64.eq (f64.ceil (f64.const -0x1.B333333333333p0)) (f64.const -0x1.0000000000000p0))
    (f64.eq (f64.ceil (f64.const -0x1.8000000000000p0)) (f64.const -0x1.0000000000000p0))
    (f64.eq (f64.ceil (f64.const 0x1.B333333333333p0)) (f64.const 0x1.0000000000000p1))

    (f64.eq (f64.convert_i32_s (i32.const 47)) (f64.const 0x1.7800000000000p5))
    (f64.eq (f64.convert_i32_s (i32.const 3824967297)) (f64.const -0x1.C03A17F000000p28))
    (f64.eq (f64.convert_i32_s (i32.const 2147484296)) (f64.const -0x1.FFFFF5E000000p30))
    (f64.eq (f64.convert_i32_s (i32.const 2147483001)) (f64.const 0x1.FFFFF5E400000p30))

    (f64.eq (f64.convert_i32_u (i32.const 2147483648)) (f64.const 0x1.0000000000000p31))
    (f64.eq (f64.convert_i32_u (i32.const 3000000000)) (f64.const 0x1.65A0BC0000000p31))

    (f64.eq (f64.div (f64.const 0x1.A000000000000p2)(f64.const 0x1.6800000000000p3)) (f64.const 0x1.27D27D27D27D2p-1))
    (f64.eq (f64.div (f64.const 0x1.A000000000000p2)(f64.const 0)) (f64.const inf))

    (f64.eq (f64.floor (f64.const 0x1.8000000000000p0)) (f64.const 0x1.0000000000000p0))
    (f64.eq (f64.floor (f64.const -0x1.B333333333333p0)) (f64.const -0x1.0000000000000p1))
    (f64.eq (f64.floor (f64.const -0x1.8000000000000p0)) (f64.const -0x1.0000000000000p1))
    (f64.eq (f64.floor (f64.const 0x1.B333333333333p0)) (f64.const 0x1.0000000000000p0))
    (i32.eq (f64.ge (f64.const 0)(f64.const 0)) (i32.const 1))
    (i32.eq (f64.ge (f64.const 0x1.0000000000000p31)(f64.const -0x1.0000000000000p31)) (i32.const 1))
    (i32.eq (f64.ge (f64.const -0x1.0000000000000p31)(f64.const 0x1.0000000000000p31)) (i32.const 0))
    (i32.eq (f64.ge (f64.const -0)(f64.const 0)) (i32.const 1))
    (i32.eq (f64.ge (f64.const nan)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.gt (f64.const 0)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.gt (f64.const 0x1.0000000000000p31)(f64.const -0x1.0000000000000p31)) (i32.const 1))
    (i32.eq (f64.gt (f64.const -0x1.0000000000000p31)(f64.const 0x1.0000000000000p31)) (i32.const 0))
    (i32.eq (f64.gt (f64.const -0)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.gt (f64.const nan)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.le (f64.const 0)(f64.const 0)) (i32.const 1))
    (i32.eq (f64.le (f64.const 0x1.0000000000000p31)(f64.const -0x1.0000000000000p31)) (i32.const 0))
    (i32.eq (f64.le (f64.const -0x1.0000000000000p31)(f64.const 0x1.0000000000000p31)) (i32.const 1))
    (i32.eq (f64.le (f64.const -0)(f64.const 0)) (i32.const 1))
    (i32.eq (f64.le (f64.const nan)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.lt (f64.const 0)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.lt (f64.const 0x1.0000000000000p31)(f64.const -0x1.0000000000000p31)) (i32.const 0))
    (i32.eq (f64.lt (f64.const -0x1.0000000000000p31)(f64.const 0x1.0000000000000p31)) (i32.const 1))
    (i32.eq (f64.lt (f64.const -0)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.lt (f64.const nan)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.ne (f64.const 0)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.ne (f64.const 0x1.0000000000000p31)(f64.const -0x1.0000000000000p31)) (i32.const 1))
    (i32.eq (f64.ne (f64.const -0)(f64.const 0)) (i32.const 0))
    (i32.eq (f64.ne (f64.const nan)(f64.const 0)) (i32.const 1))
    (f64.eq (f64.max (f64.const 0)(f64.const 0x1.0000000000000p31)) (f64.const 0x1.0000000000000p31))
    (f64.eq (f64.max (f64.const 0x1.0000000000000p31)(f64.const 0)) (f64.const 0x1.0000000000000p31))
    (f64.eq (f64.max (f64.const -0x1.0000000000000p31)(f64.const 0x1.0000000000000p31)) (f64.const 0x1.0000000000000p31))
    (f64.eq (f64.max (f64.const 0)(f64.const -0)) (f64.const 0))
    (f64.eq (f64.max (f64.const -0)(f64.const 0)) (f64.const 0))
    (f64.ne (f64.max (f64.const 0x1.0000042280000p-1023)(f64.const nan)) (f64.const 0x1.0000042280000p-1023))
    (f64.ne (f64.max (f64.const nan)(f64.const 0x1.0000042280000p-1023)) (f64.const 0x1.0000042280000p-1023))
    (f64.eq (f64.min (f64.const 0)(f64.const 0x1.0000000000000p31)) (f64.const 0))
    (f64.eq (f64.min (f64.const 0x1.0000000000000p31)(f64.const 0)) (f64.const 0))
    (f64.eq (f64.min (f64.const -0x1.0000000000000p31)(f64.const 0x1.0000000000000p31)) (f64.const -0x1.0000000000000p31))
    (f64.eq (f64.min (f64.const 0)(f64.const -0)) (f64.const -0))
    (f64.eq (f64.min (f64.const -0)(f64.const 0)) (f64.const -0))
    (f64.ne (f64.min (f64.const 0x1.0000042280000p-1023)(f64.const nan)) (f64.const 0x1.0000042280000p-1023))
    (f64.ne (f64.min (f64.const nan)(f64.const 0x1.0000042280000p-1023)) (f64.const 0x1.0000042280000p-1023))
    (f64.eq (f64.mul (f64.const 0x1.A000000000000p2)(f64.const 0x1.6800000000000p3)) (f64.const 0x1.2480000000000p6))
    (f64.eq (f64.mul (f64.const 0)(f64.const 0)) (f64.const 0))

    (f64.eq (f64.nearest (f64.const 0)) (f64.const 0))
    (f64.eq (f64.nearest (f64.const -0x1.8C010624DD2F2p6)) (f64.const -0x1.8C00000000000p6))
    (f64.eq (f64.nearest (f64.const 0x1.68F8F5C28F5C3p10)) (f64.const 0x1.6900000000000p10))
    (f64.eq (f64.neg (f64.const -0x1.00000AABBCCDDp12)) (f64.const 0x1.00000AABBCCDDp12))
    (f64.eq (f64.neg (f64.const 0x1.55555EEFF2233p342)) (f64.const -0x1.55555EEFF2233p342))
    (f64.eq (f64.neg (f64.const -0)) (f64.const 0))
    (f64.eq (f64.sqrt (f64.const 0)) (f64.const 0))
    (f64.eq (f64.sqrt (f64.const 0x1.2000000000000p3)) (f64.const 0x1.8000000000000p1))
    (f64.eq (f64.sub (f64.const 0x1.A000000000000p2)(f64.const 0x1.6800000000000p3)) (f64.const -0x1.3000000000000p2))
    (f64.eq (f64.sub (f64.const 0)(f64.const 0)) (f64.const 0))
    (f64.eq (f64.trunc (f64.const 0x1.8000000000000p0)) (f64.const 0x1.0000000000000p0))
    (f64.eq (f64.trunc (f64.const -0x1.B333333333333p0)) (f64.const -0x1.0000000000000p0))
    (f64.eq (f64.trunc (f64.const -0x1.8000000000000p0)) (f64.const -0x1.0000000000000p0))
    (f64.eq (f64.trunc (f64.const 0x1.B333333333333p0)) (f64.const 0x1.0000000000000p0))

    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    i32.and
    return
))