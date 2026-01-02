(module
  (func $test_struct_layout
    ;; Store { dest: "obj", src: ".tmp0" }
    ;; Load { dest: ".tmp1", src: "const.string.\"BadLayout created\"" }
    local.get $.tmp1
    call $println
  )
  (func $create_point (result i32)
    ;; RVO优化
    local.get $.tmp1
    return
  )
  (func $test_stack_allocation
    ;; Store { dest: "p1", src: ".tmp1" }
    ;; Store { dest: "p2", src: ".tmp2" }
    ;; Load { dest: ".tmp3", src: "const.string.\"Points created on stack\"" }
    local.get $.tmp3
    call $println
  )
  (func $main
    ;; Load { dest: ".tmp6", src: "const.string.\"All tests passed!\"" }
    local.get $.tmp6
    call $println
  )
  (export "main" (func $main))
)
