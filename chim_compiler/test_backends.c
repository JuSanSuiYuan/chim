#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>

int32_t add(int32_t a, int32_t b);
int32_t multiply(int32_t x, int32_t y);
void main(void);

int32_t add(int32_t a, int32_t b) {
    // 局部变量
    int32_t .tmp1 = a + b;
    return .tmp1; // RVO优化
}

int32_t multiply(int32_t x, int32_t y) {
    // 局部变量
    int32_t .tmp2 = x * y;
    return .tmp2; // RVO优化
}

void main(void) {
    // 局部变量
    auto .tmp5 = add(10, 20);
    int32_t result;
    result = .tmp5;
    auto .tmp8 = multiply(5, 6);
    int32_t product;
    product = .tmp8;
}

