/**
 * @file common.c
 * @brief Chim 3.1 编译器公共实现
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>

#include "common.h"

/* 内存统计 */
static size_t g_alloc_count = 0;
static size_t g_alloc_bytes = 0;

/* 初始化公共模块 */
int chim_common_init(void) {
    g_alloc_count = 0;
    g_alloc_bytes = 0;
    return 0;
}

/* 清理公共模块 */
void chim_common_cleanup(void) {
    if (g_alloc_count > 0) {
        fprintf(stderr, "警告: 内存泄漏检测: %zu 次分配未释放 (%zu bytes)\n",
            g_alloc_count, g_alloc_bytes);
    }
}

/* 内存管理 */
void* chim_alloc(size_t size) {
    if (size == 0) {
        return NULL;
    }

    void* ptr = malloc(size + sizeof(size_t));
    if (!ptr) {
        fprintf(stderr, "错误: 内存分配失败 (请求 %zu bytes)\n", size);
        chim_out_of_memory();
        return NULL;
    }

    *(size_t*)ptr = size;
    g_alloc_count++;
    g_alloc_bytes += size;

    return (char*)ptr + sizeof(size_t);
}

void* chim_realloc(void* ptr, size_t size) {
    if (!ptr) {
        return chim_alloc(size);
    }

    if (size == 0) {
        chim_free(ptr);
        return NULL;
    }

    size_t old_size = *(size_t*)((char*)ptr - sizeof(size_t));
    void* new_ptr = chim_alloc(size);

    if (new_ptr) {
        size_t copy_size = old_size < size ? old_size : size;
        memcpy(new_ptr, ptr, copy_size);
    }

    chim_free(ptr);
    return new_ptr;
}

void chim_free(void* ptr) {
    if (!ptr) return;

    size_t size = *(size_t*)((char*)ptr - sizeof(size_t));
    g_alloc_count--;
    g_alloc_bytes -= size;

    free((char*)ptr - sizeof(size_t));
}

char* chim_strdup(const char* str) {
    if (!str) return NULL;

    size_t len = strlen(str) + 1;
    char* dup = (char*)chim_alloc(len);

    if (dup) {
        memcpy(dup, str, len);
    }

    return dup;
}

/* 错误处理 */
chim_error_t* chim_error_create(chim_error_code_t code, const char* format, ...) {
    chim_error_t* err = (chim_error_t*)chim_alloc(sizeof(chim_error_t));
    if (!err) return NULL;

    va_list args;
    va_start(args, format);

    size_t len = vsnprintf(NULL, 0, format, args) + 1;
    err->message = (char*)chim_alloc(len);
    vsnprintf(err->message, len, format, args);

    va_end(args);

    err->code = code;
    err->location = (chim_location_t){NULL, 0, 0, 0};

    return err;
}

void chim_error_destroy(chim_error_t* err) {
    if (!err) return;
    chim_free(err->message);
    chim_free(err);
}

void chim_error_print(chim_diagnostics_t* diag, const chim_error_t* err) {
    if (!err || !diag) return;

    chim_diag_error(diag, "[错误 %d] %s", err->code, err->message);
}

/* 诊断输出 */
void chim_diag_init(chim_diagnostics_t* diag, FILE* output, chim_log_level_t level) {
    diag->output = output ? output : stderr;
    diag->level = level;
    diag->error_count = 0;
    diag->warning_count = 0;
    diag->strict_mode = false;
}

void chim_diag_cleanup(chim_diagnostics_t* diag) {
    CHIM_UNUSED(diag);
}

void chim_diag_set_level(chim_diagnostics_t* diag, chim_log_level_t level) {
    diag->level = level;
}

static void chim_diag_vprintf(chim_diagnostics_t* diag, const char* prefix,
    const char* format, va_list args) {
    if (!diag || !diag->output) return;

    fprintf(diag->output, "%s", prefix);

    vfprintf(diag->output, format, args);

    if (format[strlen(format) - 1] != '\n') {
        fprintf(diag->output, "\n");
    }
}

void chim_diag_error(chim_diagnostics_t* diag, const char* format, ...) {
    va_list args;
    va_start(args, format);

    chim_diag_vprintf(diag, "[错误] ", format, args);

    va_end(args);

    if (diag) {
        diag->error_count++;
    }
}

void chim_diag_warning(chim_diagnostics_t* diag, const char* format, ...) {
    if (!diag || diag->level < CHIM_LOG_WARNING) return;

    va_list args;
    va_start(args, format);

    chim_diag_vprintf(diag, "[警告] ", format, args);

    va_end(args);

    if (diag) {
        diag->warning_count++;
    }
}

void chim_diag_info(chim_diagnostics_t* diag, const char* format, ...) {
    if (!diag || diag->level < CHIM_LOG_INFO) return;

    va_list args;
    va_start(args, format);

    chim_diag_vprintf(diag, "[信息] ", format, args);

    va_end(args);
}

void chim_diag_debug(chim_diagnostics_t* diag, const char* format, ...) {
    if (!diag || diag->level < CHIM_LOG_DEBUG) return;

    va_list args;
    va_start(args, format);

    chim_diag_vprintf(diag, "[调试] ", format, args);

    va_end(args);
}

/* 位置管理 */
chim_location_t chim_location_create(const char* filename, int line, int column, int offset) {
    chim_location_t loc;
    loc.filename = filename;
    loc.line = line;
    loc.column = column;
    loc.offset = offset;
    return loc;
}

void chim_location_copy(chim_location_t* dest, const chim_location_t* src) {
    if (!dest || !src) return;
    *dest = *src;
}

bool chim_location_equal(const chim_location_t* a, const chim_location_t* b) {
    if (!a || !b) return a == b;
    return a->line == b->line && a->column == b->column;
}

/* 字符串工具 */
char* chim_sprintf(const char* format, ...) {
    va_list args;
    va_start(args, format);

    size_t len = vsnprintf(NULL, 0, format, args) + 1;
    char* result = (char*)chim_alloc(len);

    if (result) {
        vsnprintf(result, len, format, args);
    }

    va_end(args);
    return result;
}

char* chim_strcat(char* dest, size_t size, const char* src) {
    if (!dest || !src) return dest;

    size_t dest_len = strlen(dest);
    size_t src_len = strlen(src);

    if (dest_len + src_len + 1 > size) {
        return dest;
    }

    memcpy(dest + dest_len, src, src_len + 1);
    return dest;
}

char* chim_strtrim(char* str) {
    if (!str) return NULL;

    char* end;
    while (isspace(*str)) str++;

    if (*str == 0) {
        return str;
    }

    end = str + strlen(str) - 1;
    while (end > str && isspace(*end)) end--;

    end[1] = '\0';

    return str;
}

int chim_strcasecmp(const char* a, const char* b) {
#if defined(_WIN32) || defined(_WIN64)
    return stricmp(a, b);
#else
    return strcasecmp(a, b);
#endif
}

/* 文件工具 */
bool chim_file_exists(const char* path) {
    if (!path) return false;

    FILE* f = fopen(path, "r");
    if (f) {
        fclose(f);
        return true;
    }
    return false;
}

char* chim_file_read_all(const char* path, size_t* out_size) {
    FILE* f = fopen(path, "rb");
    if (!f) {
        if (out_size) *out_size = 0;
        return NULL;
    }

    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);

    if (size < 0) {
        fclose(f);
        if (out_size) *out_size = 0;
        return NULL;
    }

    char* content = (char*)malloc(size + 1);
    if (!content) {
        fclose(f);
        if (out_size) *out_size = 0;
        return NULL;
    }

    size_t read_size = fread(content, 1, size, f);
    content[read_size] = '\0';

    if (out_size) {
        *out_size = read_size;
    }

    fclose(f);
    return content;
}

bool chim_file_write_all(const char* path, const char* content, size_t size) {
    FILE* f = fopen(path, "wb");
    if (!f) {
        return false;
    }

    size_t written = fwrite(content, 1, size, f);
    fclose(f);

    return written == size;
}

char* chim_get_file_extension(const char* filename) {
    if (!filename) return NULL;

    const char* dot = strrchr(filename, '.');
    if (!dot || dot == filename) {
        return chim_strdup("");
    }

    return chim_strdup(dot + 1);
}

char* chim_get_file_basename(const char* filename) {
    if (!filename) return NULL;

    const char* slash = strrchr(filename, '/');
    if (!slash) {
        slash = strrchr(filename, '\\');
    }

    const char* name = slash ? slash + 1 : filename;
    char* basename = chim_strdup(name);

    /* 去除扩展名 */
    char* dot = strrchr(basename, '.');
    if (dot) {
        *dot = '\0';
    }

    return basename;
}

/* 编译器实例 */
chim_compiler_t* chim_compiler_create(const chim_compiler_options_t* options) {
    chim_compiler_t* compiler = (chim_compiler_t*)chim_alloc(sizeof(chim_compiler_t));
    if (!compiler) return NULL;

    compiler->options = *options;
    chim_diag_init(&compiler->diagnostics, stdout,
        options->verbose ? CHIM_LOG_VERBOSE : CHIM_LOG_ERROR);

    return compiler;
}

void chim_compiler_destroy(chim_compiler_t* compiler) {
    if (!compiler) return;

    chim_diag_cleanup(&compiler->diagnostics);
    chim_free(compiler);
}

/* 版本信息 */
void chim_version_get(int* major, int* minor, int* patch) {
    if (major) *major = CHIM_VERSION_MAJOR;
    if (minor) *minor = CHIM_VERSION_MINOR;
    if (patch) *patch = CHIM_VERSION_PATCH;
}

const char* chim_version_string(void) {
    static char version[32];
    snprintf(version, sizeof(version), "%d.%d.%d",
        CHIM_VERSION_MAJOR, CHIM_VERSION_MINOR, CHIM_VERSION_PATCH);
    return version;
}

/* 工具函数 */
void chim_print_banner(void) {
    printf("╔═══════════════════════════════════════════════════════════════╗\n");
    printf("║           Chim 3.1 编译器                                     ║\n");
    printf("║           版本: %s                                         ║\n", chim_version_string());
    printf("║           简洁 · 无歧义 · 易实现                              ║\n");
    printf("╚═══════════════════════════════════════════════════════════════╝\n\n");
}

void chim_print_usage(const char* program_name) {
    printf("用法: %s [选项] 输入文件 -o 输出文件\n", program_name);
    printf("输入 '%s --help' 获取详细使用信息\n", program_name);
}

void chim_print_version(void) {
    printf("Chim 3.1 编译器 %s\n", chim_version_string());
}
