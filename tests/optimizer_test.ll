; ModuleID = 'chim_module'
target triple = "x86_64-pc-linux-gnu"

declare void @println(i8*)
declare void @print(i8*)

define i32 @test_dead_code() {
entry:
  %x = alloca i32
  store i32 %.tmp1, i32* %x
  %.tmp4 = load i32, i32* %x
  ret i32 %.tmp4 ; RVO
}

define i32 @test_cse(i32 %a, i32 %b) {
entry:
  %.tmp6 = load i32, i32* %b
  %.tmp7 = add i32 %.tmp6, %.tmp7
  %.tmp9 = load i32, i32* %b
  %.tmp10 = add i32 %.tmp9, %.tmp10
  %y = alloca i32
  store i32 %.tmp10, i32* %y
  %.tmp12 = load i32, i32* %y
  %.tmp13 = add i32 %.tmp12, %.tmp13
  %z = alloca i32
  store i32 %.tmp13, i32* %z
  %.tmp14 = load i32, i32* %z
  ret i32 %.tmp14 ; RVO
}

define i32 @test_algebraic(i32 %x) {
entry:
  %.tmp17 = add i32 %.tmp16, %.tmp17
  %.tmp20 = mul i32 %.tmp19, %.tmp20
  %b = alloca i32
  store i32 %.tmp20, i32* %b
  %.tmp23 = mul i32 %.tmp22, %.tmp23
  %c = alloca i32
  store i32 %.tmp23, i32* %c
  %.tmp25 = load i32, i32* %b
  %.tmp26 = add i32 %.tmp25, %.tmp26
  %.tmp27 = load i32, i32* %c
  %.tmp28 = add i32 %.tmp27, %.tmp28
  %d = alloca i32
  store i32 %.tmp28, i32* %d
  %.tmp29 = load i32, i32* %d
  ret i32 %.tmp29 ; RVO
}

define i32 @test_constant_fold() {
entry:
  %.tmp32 = add i32 %.tmp31, %.tmp32
  %.tmp35 = mul i32 %.tmp34, %.tmp35
  %b = alloca i32
  store i32 %.tmp35, i32* %b
  %.tmp37 = load i32, i32* %b
  %.tmp38 = add i32 %.tmp37, %.tmp38
  %c = alloca i32
  store i32 %.tmp38, i32* %c
  %.tmp39 = load i32, i32* %c
  ret i32 %.tmp39 ; RVO
}

define i32 @test_combined(i32 %x) {
entry:
  %.tmp44 = add i32 %.tmp43, %.tmp44
  %.tmp47 = add i32 %.tmp46, %.tmp47
  %b = alloca i32
  store i32 %.tmp47, i32* %b
  %.tmp50 = mul i32 %.tmp49, %.tmp50
  %c = alloca i32
  store i32 %.tmp50, i32* %c
  %.tmp52 = load i32, i32* %b
  %.tmp53 = add i32 %.tmp52, %.tmp53
  %.tmp54 = load i32, i32* %c
  %.tmp55 = add i32 %.tmp54, %.tmp55
  %d = alloca i32
  store i32 %.tmp55, i32* %d
  %.tmp57 = load i32, i32* %d
  ret i32 %.tmp57 ; RVO
}

define i32 @main() {
entry:
  %result5 = alloca i32
  store i32 %.tmp66, i32* %result5
  %.tmp67 = load i32, i32* %result5
  ret i32 %.tmp67 ; RVO
}

