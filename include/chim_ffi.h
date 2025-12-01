#ifndef CHIM_FFI_H
#define CHIM_FFI_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ChimIRFunction {
    char* name;
    struct ChimIRArg* args;
    unsigned int arg_count;
    char* return_type; // 可为空
} ChimIRFunction;

typedef struct ChimIRArg {
    char* name;
    char* type; // 可为空
} ChimIRArg;

typedef struct ChimIRModule {
    ChimIRFunction* funcs;
    unsigned int func_count;
} ChimIRModule;

unsigned int chim_version(void);
unsigned int chim_lex(const unsigned char* input_ptr, unsigned long long input_len);

// 构建最小 IR：扫描函数定义并返回模块结构
// 返回 1 表示成功，0 表示失败
unsigned int chim_build_ir(const unsigned char* input_ptr, unsigned long long input_len, ChimIRModule* out_module);

// 释放 IR 所有分配的内存
void chim_ir_free(ChimIRModule* module);

#ifdef __cplusplus
}
#endif

#endif
