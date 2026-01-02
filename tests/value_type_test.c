// TinyCC: fast compilation mode
// Optimized for TinyCC
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>

void test_struct_layout(void);
int32_t create_point(void);
void test_stack_allocation(void);
void main(void);

void test_struct_layout(void) {
    // 局部变量
    obj = .tmp0;
    auto .tmp1 = const.string."BadLayout created";
    println(.tmp1);
}

int32_t create_point(void) {
    // 局部变量
    return .tmp1; // RVO优化
}

void test_stack_allocation(void) {
    // 局部变量
    p1 = .tmp1;
    p2 = .tmp2;
    auto .tmp3 = const.string."Points created on stack";
    println(.tmp3);
}

void main(void) {
    // 局部变量
    auto .tmp6 = const.string."All tests passed!";
    println(.tmp6);
}

