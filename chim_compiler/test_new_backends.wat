(module
  (func $add (param $a i32) (param $b i32) (result i32)
  )
  (func $main
    local.get $5
    local.get $10
    call $add
    local.set $.tmp4
  )
  (export "main" (func $main))
)
