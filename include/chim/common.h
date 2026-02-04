/**
 * @file common.h
 * @brief Chim 3.1 编译器公共定义
 *
 * 包含编译器使用的公共类型、宏和函数声明
 */

#ifndef CHIM_COMMON_H
#define CHIM_COMMON_H

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <stdarg.h>
#include <stdint.h>

/* 版本信息 */
#define CHIM_VERSION_MAJOR 3
#define CHIM_VERSION_MINOR 1
#define CHIM_VERSION_PATCH 0

/* 平台检测 */
#if defined(_WIN32) || defined(_WIN64)
    #define CHIM_PLATFORM_WINDOWS
#elif defined(__APPLE__) || defined(__MACH__)
    #define CHIM_PLATFORM_MACOS
#elif defined(__linux__)
    #define CHIM_PLATFORM_LINUX
#else
    #define CHIM_PLATFORM_UNKNOWN
#endif

/* 编译器特性 */
#define CHIM_HAS_64BIT (sizeof(void*) >= 8)

/* 常用宏 */
#define CHIM_ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))
#define CHIM_MIN(a, b) ((a) < (b) ? (a) : (b))
#define CHIM_MAX(a, b) ((a) > (b) ? (a) : (b))
#define CHIM_UNUSED(x) ((void)(x))

/* 错误码定义 */
typedef enum {
    CHIM_OK = 0,
    CHIM_ERR_GENERIC = -1,
    CHIM_ERR_OUT_OF_MEMORY = -2,
    CHIM_ERR_FILE_NOT_FOUND = -3,
    CHIM_ERR_PERMISSION_DENIED = -4,
    CHIM_ERR_INVALID_INPUT = -5,
    CHIM_ERR_SYNTAX_ERROR = -6,
    CHIM_ERR_TYPE_ERROR = -7,
    CHIM_ERR_UNDEFINED_SYMBOL = -8,
    CHIM_ERRBackend_ERROR = -9,
    CHIM_ERR_OPTIMIZATION_FAILED = -10,
} chim_error_code_t;

/* 输出级别 */
typedef enum {
    CHIM_LOG_ERROR = 0,
    CHIM_LOG_WARNING = 1,
    CHIM_LOG_INFO = 2,
    CHIM_LOG_DEBUG = 3,
    CHIM_LOG_VERBOSE = 4,
} chim_log_level_t;

/* 编译目标平台 */
typedef enum {
    CHIM_TARGET_C = 0,
    CHIM_TARGET_WASM = 1,
} chim_target_t;

/* 优化级别 */
typedef enum {
    CHIM_OPT_NONE = 0,
    CHIM_OPT_BASIC = 1,
    CHIM_OPT_STANDARD = 2,
    CHIM_OPT_AGGRESSIVE = 3,
} chim_optimization_level_t;

/* 源位置信息 */
typedef struct {
    const char* filename;
    int line;
    int column;
    int offset;
} chim_location_t;

/* 错误信息 */
typedef struct {
    chim_location_t location;
    chim_error_code_t code;
    char* message;
} chim_error_t;

/* 诊断上下文 */
typedef struct {
    FILE* output;
    chim_log_level_t level;
    int error_count;
    int warning_count;
    bool strict_mode;
} chim_diagnostics_t;

/* 编译器选项 */
typedef struct {
    const char* input_file;
    const char* output_file;
    const char* emit_c_file;
    const char* emit_ast_file;
    const char* emit_ir_file;
    chim_target_t target;
    chim_optimization_level_t opt_level;
    bool verbose;
    bool debug_info;
    bool print_ast;
    bool print_ir;
    int warning_level;
} chim_compiler_options_t;

/* 初始化公共模块 */
int chim_common_init(void);

/* 清理公共模块 */
void chim_common_cleanup(void);

/* 内存管理 */
void* chim_alloc(size_t size);
void* chim_realloc(void* ptr, size_t size);
void chim_free(void* ptr);
char* chim_strdup(const char* str);

/* 错误处理 */
chim_error_t* chim_error_create(chim_error_code_t code, const char* format, ...);
void chim_error_destroy(chim_error_t* err);
void chim_error_print(chim_diagnostics_t* diag, const chim_error_t* err);

/* 诊断输出 */
void chim_diag_init(chim_diagnostics_t* diag, FILE* output, chim_log_level_t level);
void chim_diag_cleanup(chim_diagnostics_t* diag);
void chim_diag_set_level(chim_diagnostics_t* diag, chim_log_level_t level);
void chim_diag_error(chim_diagnostics_t* diag, const char* format, ...);
void chim_diag_warning(chim_diagnostics_t* diag, const char* format, ...);
void chim_diag_info(chim_diagnostics_t* diag, const char* format, ...);
void chim_diag_debug(chim_diagnostics_t* diag, const char* format, ...);

/* 位置管理 */
chim_location_t chim_location_create(const char* filename, int line, int column, int offset);
void chim_location_copy(chim_location_t* dest, const chim_location_t* src);
bool chim_location_equal(const chim_location_t* a, const chim_location_t* b);

/* 字符串工具 */
char* chim_sprintf(const char* format, ...);
char* chim_strcat(char* dest, size_t size, const char* src);
char* chim_strtrim(char* str);
int chim_strcasecmp(const char* a, const char* b);

/* 文件工具 */
bool chim_file_exists(const char* path);
char* chim_file_read_all(const char* path, size_t* out_size);
bool chim_file_write_all(const char* path, const char* content, size_t size);
char* chim_get_file_extension(const char* filename);
char* chim_get_file_basename(const char* filename);

/* 编译器实例上下文 */
typedef struct {
    chim_compiler_options_t options;
    chim_diagnostics_t diagnostics;
    void* frontend_context;
    void* middleend_context;
    void* backend_context;
} chim_compiler_t;

/* 初始化编译器实例 */
chim_compiler_t* chim_compiler_create(const chim_compiler_options_t* options);
void chim_compiler_destroy(chim_compiler_t* compiler);

/* 版本信息获取 */
void chim_version_get(int* major, int* minor, int* patch);
const char* chim_version_string(void);

/* 工具函数 */
void chim_print_banner(void);
void chim_print_usage(const char* program_name);
void chim_print_version(void);

#endif /* CHIM_COMMON_H */
