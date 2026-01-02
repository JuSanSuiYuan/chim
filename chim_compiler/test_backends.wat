(module
  (func $add (param $a i32) (param $b i32) (result i32)
    ;; Load { dest: ".tmp2", src: "b" }
    local.get $.tmp2
    local.get $.tmp3
    i32.add
    local.set $.tmp3
    ;; RVO优化
    local.get $.tmp3
    return
  )
  (func $multiply (param $x i32) (param $y i32) (result i32)
    ;; Load { dest: ".tmp5", src: "y" }
    local.get $.tmp5
    local.get $.tmp6
    i32.mul
    local.set $.tmp6
    ;; RVO优化
    local.get $.tmp6
    return
  )
  (func $main
    local.get $.tmp7
    local.get $.tmp8
    call $add
    local.set $.tmp9
    local.get $.tmp10
    local.get $.tmp11
    call $multiply
    local.set $.tmp12
  )
  (export "main" (func $main))
)
