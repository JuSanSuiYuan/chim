/**
 * @file codegen.h
 * @brief Chim 3.1 代码生成器接口
 */

#ifndef CHIM_CODEGEN_H
#define CHIM_CODEGEN_H

#include "ir.h"
#include "common.h"

/* 代码生成器目标 */
typedef enum {
    CHIM_CODEGEN_TARGET_C,
    CHIM_CODEGEN_TARGET_WASM,
} chim_codegen_target_t;

/* 代码生成器配置 */
typedef struct {
    chim_codegen_target_t target;
    bool generate_debug_info;
    bool generate_line_info;
    bool enable_optimizations;
    const char* output_filename;
    const char* module_name;
    const char* runtime_path;
} chim_codegen_config_t;

/* 代码生成器上下文 */
typedef struct {
    chim_ir_module_t* module;
    chim_codegen_config_t config;
    FILE* output;
    int indent_level;
    int temp_var_count;
    int label_count;
    bool is_in_function;
    chim_ir_function_t* current_function;
    chim_ir_basic_block_t* current_block;
} chim_codegen_t;

/* 代码生成器创建 */
chim_codegen_t* chim_codegen_create(chim_ir_module_t* module);
chim_codegen_t* chim_codegen_create_with_config(chim_ir_module_t* module,
    const chim_codegen_config_t* config);
void chim_codegen_destroy(chim_codegen_t* codegen);

/* 代码生成 */
bool chim_codegen_emit(chim_codegen_t* codegen, const char* filename);
bool chim_codegen_emit_to_file(chim_codegen_t* codegen, FILE* file);
bool chim_codegen_emit_to_string(chim_codegen_t* codegen, char** out_string, size_t* out_size);

/* 目标配置 */
void chim_codegen_config_init(chim_codegen_config_t* config);
void chim_codegen_config_set_target(chim_codegen_config_t* config, chim_codegen_target_t target);

/* 辅助函数 */
void chim_codegen_emit_line(chim_codegen_t* codegen, const char* format, ...);
void chim_codegen_emit_block_start(chim_codegen_t* codegen);
void chim_codegen_emit_block_end(chim_codegen_t* codegen);
void chim_codegen_indent(chim_codegen_t* codegen);
void chim_codegen_dedent(chim_codegen_t* codegen);
void chim_codegen_newline(chim_codegen_t* codegen);

/* 代码生成 Pass */
typedef bool (*chim_codegen_pass_t)(chim_codegen_t* codegen, void* data);

/* 注册自定义 Pass */
void chim_codegen_register_pre_pass(chim_codegen_t* codegen, chim_codegen_pass_t pass, void* data);
void chim_codegen_register_post_pass(chim_codegen_t* codegen, chim_codegen_pass_t pass, void* data);

/* 获取生成代码统计 */
typedef struct {
    size_t num_lines;
    size_t num_characters;
    size_t num_includes;
    size_t num_functions;
} chim_codegen_stats_t;

chim_codegen_stats_t chim_codegen_get_stats(chim_codegen_t* codegen);

#endif /* CHIM_CODEGEN_H */
