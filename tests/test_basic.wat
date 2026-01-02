(module
  ;; Chim Programming Language - WebAssembly Output
  ;; Generated 3 functions, 0 structs

  ;; Memory configuration
  (memory (export "memory") 1)
  (global (export "heap_base") i32 (i32.const 0))

  ;; Built-in functions
  (func $print (param $msg i32)
    (call $puts (local.get $msg))
  )
  (import "env" "puts" (func $puts (param i32)))
  (func $println (param $msg i32)
    (call $puts (local.get $msg))
    (call $putchar (i32.const 10))
  )
  (import "env" "putchar" (func $putchar (param i32)))
  (func $len (param $str i32) (result i32)
    (local $len i32)
    (local $i i32)
    (block $exit
      (loop $loop
        (br_if $exit (i32.eqz (i32.load8_u (local.get $i))))
        (local.set $len (i32.add (local.get $len) (i32.const 1)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop)
      )
    )
    (local.get $len)
  )

  (export "add" (func $add))
  (func $add (param i32 i32) (result i32)
  (local.set .tmp1 (local.get a))
  (local.set .tmp2 (local.get b))
  (local.set .tmp3 (i32.add (local.get .tmp2) (local.get .tmp3)))
  )

  (export "multiply" (func $multiply))
  (func $multiply (param i32 i32) (result i32)
  (local.set .tmp4 (local.get x))
  (local.set .tmp5 (local.get y))
  (local.set .tmp6 (i32.mul (local.get .tmp5) (local.get .tmp6)))
  )

  (export "main" (func $main))
  (func $main (param ) (result [])
    (local result i32)
    (local product i32)
  (local.set .tmp9 (call $add (local.get .tmp7) (local.get .tmp8)))
  (local result i32)
  (local.set result (local.get .tmp9))
  (local.set .tmp12 (call $multiply (local.get .tmp10) (local.get .tmp11)))
  (local product i32)
  (local.set product (local.get .tmp12))
  )

)
