; ModuleID = 'chim_module'
target triple = "x86_64-pc-linux-gnu"

declare void @println(i8*)
declare void @print(i8*)

define i32 @add(i32 %a, i32 %b) {
entry:
  %.tmp1 = add i32 %a, %b
  ret i32 %.tmp1 ; RVO
}

define i32 @subtract(i32 %a, i32 %b) {
entry:
  %.tmp2 = sub i32 %a, %b
  ret i32 %.tmp2 ; RVO
}

define i32 @multiply(i32 %a, i32 %b) {
entry:
  %.tmp3 = mul i32 %a, %b
  ret i32 %.tmp3 ; RVO
}

define i32 @divide(i32 %a, i32 %b) {
entry:
  %.tmp4 = sdiv i32 %a, %b
  ret i32 %.tmp4 ; RVO
}

define i32 @modulo(i32 %a, i32 %b) {
entry:
  ; Mod { dest: ".tmp5", left: "a", right: "b" }
  ret i32 %.tmp5 ; RVO
}

define void @main() {
entry:
  %.tmp8 = call i32 @add(i32 %10, i32 %20)
  %.tmp11 = call i32 @subtract(i32 %30, i32 %15)
  %.tmp14 = call i32 @multiply(i32 %5, i32 %6)
  %.tmp17 = call i32 @divide(i32 %100, i32 %10)
  %.tmp20 = call i32 @modulo(i32 %17, i32 %5)
  ret void
}

