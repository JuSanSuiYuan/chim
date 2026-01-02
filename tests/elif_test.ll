; ModuleID = 'chim_module'
target triple = "x86_64-pc-linux-gnu"

declare void @println(i8*)
declare void @print(i8*)

define i8* @test_grade(i32 %score) {
entry:
  ; Ge { dest: ".tmp3", left: ".tmp2", right: ".tmp3" }
  br i1 %.tmp3, label %.L1, label %.L2
.L1:
  %.tmp4 = load i32, i32* %const.string."优秀"
  ret i32 %.tmp4 ; RVO
  br label %.L3
.L2:
  ; Ge { dest: ".tmp7", left: ".tmp6", right: ".tmp7" }
  br i1 %.tmp7, label %.L4, label %.L5
.L4:
  %.tmp8 = load i32, i32* %const.string."良好"
  ret i32 %.tmp8 ; RVO
  br label %.L6
.L5:
  ; Ge { dest: ".tmp11", left: ".tmp10", right: ".tmp11" }
  br i1 %.tmp11, label %.L7, label %.L8
.L7:
  %.tmp12 = load i32, i32* %const.string."中等"
  ret i32 %.tmp12 ; RVO
  br label %.L9
.L8:
  ; Ge { dest: ".tmp15", left: ".tmp14", right: ".tmp15" }
  br i1 %.tmp15, label %.L10, label %.L11
.L10:
  %.tmp16 = load i32, i32* %const.string."及格"
  ret i32 %.tmp16 ; RVO
  br label %.L12
.L11:
  %.tmp17 = load i32, i32* %const.string."不及格"
  ret i32 %.tmp17 ; RVO
  br label %.L12
.L12:
  br label %.L9
.L9:
  br label %.L6
.L6:
  br label %.L3
.L3:
}

define i32 @test_mixed(i32 %x) {
entry:
  ; Gt { dest: ".tmp20", left: ".tmp19", right: ".tmp20" }
  br i1 %.tmp20, label %.L13, label %.L14
.L13:
  ret i32 %.tmp21 ; RVO
  br label %.L15
.L14:
  ; Gt { dest: ".tmp24", left: ".tmp23", right: ".tmp24" }
  br i1 %.tmp24, label %.L16, label %.L17
.L16:
  ret i32 %.tmp25 ; RVO
  br label %.L18
.L17:
  ; Gt { dest: ".tmp28", left: ".tmp27", right: ".tmp28" }
  br i1 %.tmp28, label %.L19, label %.L20
.L19:
  ret i32 %.tmp29 ; RVO
  br label %.L21
.L20:
  ret i32 %.tmp30 ; RVO
  br label %.L21
.L21:
  br label %.L18
.L18:
  br label %.L15
.L15:
}

define i32 @main() {
entry:
  %.tmp32 = call i32 @test_grade(i32 %.tmp31)
  %.tmp34 = call i32 @test_grade(i32 %.tmp33)
  %.tmp36 = call i32 @test_grade(i32 %.tmp35)
  %.tmp38 = call i32 @test_mixed(i32 %.tmp37)
  %result = alloca i32
  store i32 %.tmp38, i32* %result
  %.tmp39 = load i32, i32* %result
  ret i32 %.tmp39 ; RVO
}

