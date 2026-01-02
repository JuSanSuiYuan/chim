; ModuleID = 'chim_module'
target triple = "x86_64-pc-linux-gnu"

declare void @println(i8*)
declare void @print(i8*)

define void @test_struct_layout() {
entry:
  store i32 %.tmp0, i32* %obj
  %.tmp1 = load i32, i32* %const.string."BadLayout created"
  call void @println(i32 %.tmp1)
  ret void
}

define i32 @create_point() {
entry:
  ret i32 %.tmp1 ; RVO
}

define void @test_stack_allocation() {
entry:
  store i32 %.tmp1, i32* %p1
  store i32 %.tmp2, i32* %p2
  %.tmp3 = load i32, i32* %const.string."Points created on stack"
  call void @println(i32 %.tmp3)
  ret void
}

define void @main() {
entry:
  %.tmp6 = load i32, i32* %const.string."All tests passed!"
  call void @println(i32 %.tmp6)
  ret void
}

