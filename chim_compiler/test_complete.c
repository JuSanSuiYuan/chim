#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>

int32_t add(int32_t a, int32_t b);
int32_t subtract(int32_t a, int32_t b);
int32_t multiply(int32_t a, int32_t b);
int32_t divide(int32_t a, int32_t b);
int32_t modulo(int32_t a, int32_t b);
void main(void);

int32_t add(int32_t a, int32_t b) {
    // 局部变量
    int32_t .tmp1 = a + b;
    return .tmp1; // RVO优化
}

int32_t subtract(int32_t a, int32_t b) {
    // 局部变量
    int32_t .tmp2 = a - b;
    return .tmp2; // RVO优化
}

int32_t multiply(int32_t a, int32_t b) {
    // 局部变量
    int32_t .tmp3 = a * b;
    return .tmp3; // RVO优化
}

int32_t divide(int32_t a, int32_t b) {
    // 局部变量
    int32_t .tmp4 = a / b;
    return .tmp4; // RVO优化
}

int32_t modulo(int32_t a, int32_t b) {
    // 局部变量
    int32_t .tmp5 = a % b;
    return .tmp5; // RVO优化
}

void main(void) {
    // 局部变量
    auto .tmp8 = add(10, 20);
    auto .tmp11 = subtract(30, 15);
    auto .tmp14 = multiply(5, 6);
    auto .tmp17 = divide(100, 10);
    auto .tmp20 = modulo(17, 5);
}

