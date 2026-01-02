(module
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
    local.set $.tmp1
    ;; RVO优化
    local.get $.tmp1
    return
  )
  (func $subtract (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.sub
    local.set $.tmp2
    ;; RVO优化
    local.get $.tmp2
    return
  )
  (func $multiply (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
    local.set $.tmp3
    ;; RVO优化
    local.get $.tmp3
    return
  )
  (func $divide (param $a i32) (param $b i32) (result i32)
    ;; Div { dest: ".tmp4", left: "a", right: "b" }
    ;; RVO优化
    local.get $.tmp4
    return
  )
  (func $modulo (param $a i32) (param $b i32) (result i32)
    ;; Mod { dest: ".tmp5", left: "a", right: "b" }
    ;; RVO优化
    local.get $.tmp5
    return
  )
  (func $main
    local.get $10
    local.get $20
    call $add
    local.set $.tmp8
    local.get $30
    local.get $15
    call $subtract
    local.set $.tmp11
    local.get $5
    local.get $6
    call $multiply
    local.set $.tmp14
    local.get $100
    local.get $10
    call $divide
    local.set $.tmp17
    local.get $17
    local.get $5
    call $modulo
    local.set $.tmp20
  )
  (export "main" (func $main))
)
