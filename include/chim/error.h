/**
 * @file error.h
 * @brief Chim 3.1 错误处理定义
 */

#ifndef CHIM_ERROR_H
#define CHIM_ERROR_H

#include "common.h"

/* 错误处理模块初始化 */
int chim_error_init(void);
void chim_error_cleanup(void);

/* 错误格式化 */
char* chim_error_format(chim_error_code_t code, const char* format, va_list args);
char* chim_error_format_simple(chim_error_code_t code, const char* message);

/* 语法错误处理 */
typedef struct {
    chim_location_t location;
    chim_token_type_t expected;
    chim_token_type_t found;
    const char* context;
} chim_syntax_error_t;

void chim_syntax_error_report(chim_diagnostics_t* diag,
    const chim_location_t* location,
    const char* expected_token,
    const char* found_token,
    const char* context_message);

/* 类型错误处理 */
typedef struct {
    const char* expected_type;
    const char* found_type;
    const char* operation;
} chim_type_error_t;

void chim_type_error_report(chim_diagnostics_t* diag,
    const chim_location_t* location,
    const char* expected_type,
    const char* found_type,
    const char* operation);

/* 未定义符号错误 */
void chim_undefined_symbol_error(chim_diagnostics_t* diag,
    const chim_location_t* location,
    const char* symbol_name);

/* 重复定义错误 */
void chim_redefinition_error(chim_diagnostics_t* diag,
    const chim_location_t* location,
    const char* symbol_name,
    const chim_location_t* original_location);

/* 警告处理 */
typedef enum {
    CHIM_WARNING_UNUSED_VARIABLE,
    CHIM_WARNING_UNREACHABLE_CODE,
    CHIM_WARNING_DEPRECATED,
} chim_warning_type_t;

void chim_warning_report(chim_diagnostics_t* diag,
    chim_warning_type_t type,
    const chim_location_t* location,
    const char* message);

/* 致命错误 */
void chim_fatal_error(chim_error_code_t code, const char* format, ...);

/* 内存错误 */
void chim_out_of_memory(void);

/* 内部错误 */
void chim_internal_error(const char* file, int line, const char* function,
    const char* format, ...);

#define CHIM_INTERNAL_ERROR(...) \
    chim_internal_error(__FILE__, __LINE__, __func__, __VA_ARGS__)

/* 错误恢复策略 */
typedef enum {
    CHIM_RECOVERY_NONE,
    CHIM_RECOVERY_PANIC,
    CHIM_RECOVERY_SYNCHRONIZE,
} chim_error_recovery_mode_t;

/* 错误恢复上下文 */
typedef struct {
    chim_error_recovery_mode_t mode;
    chim_token_type_t* sync_tokens;
    size_t num_sync_tokens;
    bool in_error_state;
} chim_error_recovery_context_t;

void chim_error_recovery_init(chim_error_recovery_context_t* ctx,
    chim_error_recovery_mode_t mode);
void chim_error_recovery_cleanup(chim_error_recovery_context_t* ctx);
bool chim_error_recovery_synchronize(chim_token_array_t* tokens,
    size_t current_index,
    chim_error_recovery_context_t* ctx);

#endif /* CHIM_ERROR_H */
