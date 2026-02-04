/**
 * @file c_codegen.h
 * @brief Chim 3.1 C 代码生成器头文件
 */

#ifndef CHIM_C_CODEGEN_H
#define CHIM_C_CODEGEN_H

#include "codegen.h"

/* C 代码生成器特有配置 */
typedef struct {
    chim_codegen_config_t base;
    bool generate_header;
    const char* header_filename;
    bool generate_main_wrapper;
    bool use_stdlib;
    bool enable_exceptions;
    int c_standard;  /* C89, C99, C11, C17 */
} chim_c_codegen_config_t;

/* C 代码生成器创建 */
chim_codegen_t* chim_c_codegen_create(chim_ir_module_t* module);
chim_codegen_t* chim_c_codegen_create_with_config(chim_ir_module_t* module,
    const chim_c_codegen_config_t* config);
void chim_c_codegen_destroy(chim_codegen_t* codegen);

/* C 特有配置 */
void chim_c_codegen_config_init(chim_c_codegen_config_t* config);
void chim_c_codegen_config_set_standard(chim_c_codegen_config_t* config, int standard);
void chim_c_codegen_config_enable_header(chim_c_codegen_config_t* config, bool enable,
    const char* filename);
void chim_c_codegen_config_enable_main(chim_c_codegen_config_t* config, bool enable);

/* 运行时支持 */
bool chim_c_codegen_emit_runtime(chim_codegen_t* codegen);
bool chim_c_codegen_emit_runtime_header(chim_codegen_t* codegen);

/* 内置类型映射 */
const char* chim_c_codegen_map_type(chim_ir_type_t* type);
const char* chim_c_codegen_map_type_name(const char* chim_type);

/* 内置函数声明 */
void chim_c_codegen_emit_builtin_decls(chim_codegen_t* codegen);

/* 头文件生成 */
bool chim_c_codegen_emit_header(chim_codegen_t* codegen, const char* filename);

/* 内存管理代码 */
void chim_c_codegen_emit_memory_funcs(chim_codegen_t* codegen);

/* 列表操作代码 */
void chim_c_codegen_emit_list_funcs(chim_codegen_t* codegen);

/* 字符串操作代码 */
void chim_c_codegen_emit_string_funcs(chim_codegen_t* codegen);

/* 打印函数代码 */
void chim_c_codegen_emit_print_funcs(chim_codegen_t* codegen);

#endif /* CHIM_C_CODEGEN_H */
