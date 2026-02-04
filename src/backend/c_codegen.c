/**
 * @file c_codegen.c
 * @brief Chim 3.1 C 代码生成器实现
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

#include "c_codegen.h"
#include "ir.h"
#include "ast.h"
#include "common.h"

#define INDENT_WIDTH 4

typedef struct {
    FILE* output;
    int indent_level;
    chim_c_codegen_config_t* config;
    chim_ir_module_t* module;
    int temp_counter;
    int label_counter;
} chim_c_codegen_context_t;

static void chim_c_codegen_indent(chim_c_codegen_context_t* ctx) {
    for (int i = 0; i < ctx->indent_level * INDENT_WIDTH; i++) {
        fprintf(ctx->output, " ");
    }
}

static void chim_c_codegen_write(chim_c_codegen_context_t* ctx, const char* str) {
    fprintf(ctx->output, "%s", str);
}

static void chim_c_codegen_write_indented(chim_c_codegen_context_t* ctx, const char* str) {
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, str);
}

static void chim_c_codegen_newline(chim_c_codegen_context_t* ctx) {
    fprintf(ctx->output, "\n");
}

static const char* chim_c_codegen_map_type_internal(chim_ir_type_t* type) {
    if (!type) return "void";

    switch (type->kind) {
        case CHIR_TYPE_VOID:
            return "void";
        case CHIR_TYPE_BOOL:
            return "bool";
        case CHIR_TYPE_INT8:
            return "int8_t";
        case CHIR_TYPE_INT16:
            return "int16_t";
        case CHIR_TYPE_INT32:
            return "int32_t";
        case CHIR_TYPE_INT64:
            return "int64_t";
        case CHIR_TYPE_UINT8:
            return "uint8_t";
        case CHIR_TYPE_UINT16:
            return "uint16_t";
        case CHIR_TYPE_UINT32:
            return "uint32_t";
        case CHIR_TYPE_UINT64:
            return "uint64_t";
        case CHIR_TYPE_FLOAT32:
            return "float";
        case CHIR_TYPE_FLOAT64:
            return "double";
        case CHIR_TYPE_STRING:
            return "chim_string";
        case CHIR_TYPE_LIST:
            return "chim_list";
        case CHIR_TYPE_FN:
            return "chim_fn";
        case CHIR_TYPE_PTR:
            return "void*";
        case CHIR_TYPE_STRUCT:
            return type->name ? type->name : "struct chim_struct";
        case CHIR_TYPE_ENUM:
            return "int";
        default:
            return "void";
    }
}

static const char* chim_c_codegen_get_temp_name(chim_c_codegen_context_t* ctx) {
    char* name = chim_sprintf("__temp_%d", ctx->temp_counter++);
    return name;
}

static const char* chim_c_codegen_get_label_name(chim_c_codegen_context_t* ctx) {
    char* name = chim_sprintf("__label_%d", ctx->label_counter++);
    return name;
}

static void chim_c_codegen_emit_type_aliases(chim_c_codegen_context_t* ctx) {
    chim_c_codegen_write(ctx, "/* Chim 运行时类型定义 */\n\n");

    chim_c_codegen_write(ctx, "#ifndef CHIM_TYPES_H\n");
    chim_c_codegen_write(ctx, "#define CHIM_TYPES_H\n\n");

    chim_c_codegen_write(ctx, "#include <stdbool.h>\n");
    chim_c_codegen_write(ctx, "#include <stdint.h>\n");
    chim_c_codegen_write(ctx, "#include <stddef.h>\n\n");

    chim_c_codegen_write(ctx, "/* 基础类型 */\n");
    chim_c_codegen_write(ctx, "typedef int8_t   chim_int8_t;\n");
    chim_c_codegen_write(ctx, "typedef int16_t  chim_int16_t;\n");
    chim_c_codegen_write(ctx, "typedef int32_t  chim_int32_t;\n");
    chim_c_codegen_write(ctx, "typedef int64_t  chim_int64_t;\n");
    chim_c_codegen_write(ctx, "typedef uint8_t  chim_uint8_t;\n");
    chim_c_codegen_write(ctx, "typedef uint16_t chim_uint16_t;\n");
    chim_c_codegen_write(ctx, "typedef uint32_t chim_uint32_t;\n");
    chim_c_codegen_write(ctx, "typedef uint64_t chim_uint64_t;\n");
    chim_c_codegen_write(ctx, "typedef float    chim_float32_t;\n");
    chim_c_codegen_write(ctx, "typedef double   chim_float64_t;\n\n");

    chim_c_codegen_write(ctx, "/* 字符串类型 */\n");
    chim_c_codegen_write(ctx, "typedef struct {\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "const char* data;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "size_t length;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "size_t capacity;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "bool is_literal;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "} chim_string_t;\n\n");

    chim_c_codegen_write(ctx, "/* 列表类型 */\n");
    chim_c_codegen_write(ctx, "typedef struct chim_list_node {\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "void* data;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "struct chim_list_node* next;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "struct chim_list_node* prev;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "} chim_list_node_t;\n\n");

    chim_c_codegen_write(ctx, "typedef struct {\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "chim_list_node_t* head;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "chim_list_node_t* tail;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "size_t length;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "size_t element_size;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "void (*free_fn)(void*);\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "} chim_list_t;\n\n");

    chim_c_codegen_write(ctx, "/* 函数指针类型 */\n");
    chim_c_codegen_write(ctx, "typedef void (*chim_fn_t)(void);\n\n");

    chim_c_codegen_write(ctx, "/* 选项类型 */\n");
    chim_c_codegen_write(ctx, "typedef struct {\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "bool is_some;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "union {\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "void* value;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "int64_t int_value;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "double float_value;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "};\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "} chim_option_t;\n\n");

    chim_c_codegen_write(ctx, "/* 结果类型 */\n");
    chim_c_codegen_write(ctx, "typedef struct {\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "bool is_ok;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "union {\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "void* ok_value;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "int64_t error_code;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "const char* error_msg;\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "};\n");
    chim_c_codegen_indent(ctx);
    chim_c_codegen_write(ctx, "} chim_result_t;\n\n");

    chim_c_codegen_write(ctx, "#endif /* CHIM_TYPES_H */\n\n");
}

static void chim_c_codegen_emit_runtime_functions(chim_c_codegen_context_t* ctx) {
    chim_c_codegen_write(ctx, "/* Chim 运行时函数实现 */\n\n");

    chim_c_codegen_write(ctx, "#include \"chim_types.h\"\n");
    chim_c_codegen_write(ctx, "#include \"chim_runtime.h\"\n");
    chim_c_codegen_write(ctx, "#include <string.h>\n");
    chim_c_codegen_write(ctx, "#include <stdlib.h>\n");
    chim_c_codegen_write(ctx, "#include <stdio.h>\n\n");

    chim_c_codegen_write(ctx, "/* 字符串操作函数 */\n");
    chim_c_codegen_write_indented(ctx, "chim_string_t chim_string_create(const char* data) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_string_t str;\n");
    chim_c_codegen_write_indented(ctx, "str.is_literal = false;\n");
    chim_c_codegen_write_indented(ctx, "str.length = strlen(data);\n");
    chim_c_codegen_write_indented(ctx, "str.capacity = str.length + 1;\n");
    chim_c_codegen_write_indented(ctx, "str.data = (char*)malloc(str.capacity);\n");
    chim_c_codegen_write_indented(ctx, "memcpy((void*)str.data, data, str.capacity);\n");
    chim_c_codegen_write_indented(ctx, "return str;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "chim_string_t chim_string_from_literal(const char* data) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_string_t str;\n");
    chim_c_codegen_write_indented(ctx, "str.is_literal = true;\n");
    chim_c_codegen_write_indented(ctx, "str.data = data;\n");
    chim_c_codegen_write_indented(ctx, "str.length = strlen(data);\n");
    chim_c_codegen_write_indented(ctx, "str.capacity = 0;\n");
    chim_c_codegen_write_indented(ctx, "return str;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_string_destroy(chim_string_t* str) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "if (!str->is_literal && str->data) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "free((void*)str->data);\n");
    chim_c_codegen_write_indented(ctx, "str->data = NULL;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "chim_string_t chim_string_concat(chim_string_t* a, chim_string_t* b) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_string_t result;\n");
    chim_c_codegen_write_indented(ctx, "result.is_literal = false;\n");
    chim_c_codegen_write_indented(ctx, "result.length = a->length + b->length;\n");
    chim_c_codegen_write_indented(ctx, "result.capacity = result.length + 1;\n");
    chim_c_codegen_write_indented(ctx, "result.data = (char*)malloc(result.capacity);\n");
    chim_c_codegen_write_indented(ctx, "memcpy((void*)result.data, a->data, a->length);\n");
    chim_c_codegen_write_indented(ctx, "memcpy((void*)(result.data + a->length), b->data, b->length);\n");
    chim_c_codegen_write_indented(ctx, "((char*)result.data)[result.length] = '\\0';\n");
    chim_c_codegen_write_indented(ctx, "return result;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write(ctx, "/* 列表操作函数 */\n");
    chim_c_codegen_write_indented(ctx, "chim_list_t chim_list_create(size_t element_size) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_list_t list;\n");
    chim_c_codegen_write_indented(ctx, "list.head = NULL;\n");
    chim_c_codegen_write_indented(ctx, "list.tail = NULL;\n");
    chim_c_codegen_write_indented(ctx, "list.length = 0;\n");
    chim_c_codegen_write_indented(ctx, "list.element_size = element_size;\n");
    chim_c_codegen_write_indented(ctx, "list.free_fn = NULL;\n");
    chim_c_codegen_write_indented(ctx, "return list;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_list_push(chim_list_t* list, void* element) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_list_node_t* node = (chim_list_node_t*)malloc(sizeof(chim_list_node_t));\n");
    chim_c_codegen_write_indented(ctx, "node->data = malloc(list->element_size);\n");
    chim_c_codegen_write_indented(ctx, "memcpy(node->data, element, list->element_size);\n");
    chim_c_codegen_write_indented(ctx, "node->next = NULL;\n");
    chim_c_codegen_write_indented(ctx, "node->prev = list->tail;\n\n");

    chim_c_codegen_write_indented(ctx, "if (list->tail) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "list->tail->next = node;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n");
    chim_c_codegen_write_indented(ctx, "list->tail = node;\n\n");

    chim_c_codegen_write_indented(ctx, "if (!list->head) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "list->head = node;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "list->length++;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void* chim_list_get(chim_list_t* list, size_t index) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "if (index >= list->length) return NULL;\n");
    chim_c_codegen_write_indented(ctx, "chim_list_node_t* current = list->head;\n");
    chim_c_codegen_write_indented(ctx, "for (size_t i = 0; i < index; i++) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "current = current->next;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n");
    chim_c_codegen_write_indented(ctx, "return current->data;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_list_destroy(chim_list_t* list) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_list_node_t* current = list->head;\n");
    chim_c_codegen_write_indented(ctx, "while (current) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_list_node_t* next = current->next;\n");
    chim_c_codegen_write_indented(ctx, "if (list->free_fn) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "list->free_fn(current->data);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n");
    chim_c_codegen_write_indented(ctx, "free(current->data);\n");
    chim_c_codegen_write_indented(ctx, "free(current);\n");
    chim_c_codegen_write_indented(ctx, "current = next;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n");
    chim_c_codegen_write_indented(ctx, "list->head = NULL;\n");
    chim_c_codegen_write_indented(ctx, "list->tail = NULL;\n");
    chim_c_codegen_write_indented(ctx, "list->length = 0;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write(ctx, "/* 打印函数 */\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_int(int64_t value) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "printf(\"%lld\", (long long)value);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_print_float(double value) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "printf(\"%g\", value);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_print_string(chim_string_t* str) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "printf(\"%s\", str->data);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_print_bool(bool value) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "printf(\"%s\", value ? \"true\" : \"false\");\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_print_newline(void) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "printf(\"\\n\");\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write(ctx, "/* 内存分配函数 */\n");
    chim_c_codegen_write_indented(ctx, "void* chim_alloc(size_t size) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "return malloc(size);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_free(void* ptr) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "if (ptr) free(ptr);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void* chim_realloc(void* ptr, size_t size) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "return realloc(ptr, size);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_memset(void* ptr, int value, size_t size) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "memset(ptr, value, size);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "void chim_memcpy(void* dest, void* src, size_t size) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "memcpy(dest, src, size);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "int chim_memcmp(void* a, void* b, size_t size) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "return memcmp(a, b, size);\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write(ctx, "/* 选项类型函数 */\n");
    chim_c_codegen_write_indented(ctx, "chim_option_t chim_option_some(void* value) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_option_t opt;\n");
    chim_c_codegen_write_indented(ctx, "opt.is_some = true;\n");
    chim_c_codegen_write_indented(ctx, "opt.value = value;\n");
    chim_c_codegen_write_indented(ctx, "return opt;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "chim_option_t chim_option_none(void) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_option_t opt;\n");
    chim_c_codegen_write_indented(ctx, "opt.is_some = false;\n");
    chim_c_codegen_write_indented(ctx, "opt.value = NULL;\n");
    chim_c_codegen_write_indented(ctx, "return opt;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write(ctx, "/* 结果类型函数 */\n");
    chim_c_codegen_write_indented(ctx, "chim_result_t chim_result_ok(void* value) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_result_t res;\n");
    chim_c_codegen_write_indented(ctx, "res.is_ok = true;\n");
    chim_c_codegen_write_indented(ctx, "res.ok_value = value;\n");
    chim_c_codegen_write_indented(ctx, "return res;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");

    chim_c_codegen_write_indented(ctx, "chim_result_t chim_result_error(const char* message) {\n");
    ctx->indent_level++;
    chim_c_codegen_write_indented(ctx, "chim_result_t res;\n");
    chim_c_codegen_write_indented(ctx, "res.is_ok = false;\n");
    chim_c_codegen_write_indented(ctx, "res.error_msg = message;\n");
    chim_c_codegen_write_indented(ctx, "return res;\n");
    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");
}

static void chim_c_codegen_emit_function(chim_c_codegen_context_t* ctx, chim_ir_function_t* func) {
    if (!func || !func->name) return;

    const char* return_type = chim_c_codegen_map_type_internal(&func->return_type);
    chim_c_codegen_write_indented(ctx, return_type);
    chim_c_codegen_write(ctx, " ");
    chim_c_codegen_write(ctx, func->name);
    chim_c_codegen_write(ctx, "(");

    bool first_param = true;
    for (size_t i = 0; i < func->param_count; i++) {
        if (!first_param) {
            chim_c_codegen_write(ctx, ", ");
        }
        first_param = false;

        const char* param_type = chim_c_codegen_map_type_internal(&func->param_types[i]);
        chim_c_codegen_write(ctx, param_type);
        chim_c_codegen_write(ctx, " ");
        chim_c_codegen_write(ctx, func->param_names[i]);
    }

    chim_c_codegen_write(ctx, ") {\n");
    ctx->indent_level++;

    chim_ir_instruction_t* instr = func->instructions;
    while (instr) {
        chim_c_codegen_indent(ctx);

        switch (instr->opcode) {
            case CHIR_OP_CONST:
                if (instr->result_type.kind == CHIR_TYPE_FLOAT64) {
                    fprintf(ctx->output, "double %s = %g;\n", instr->result.name, instr->const_value.float_value);
                } else if (instr->result_type.kind == CHIR_TYPE_FLOAT32) {
                    fprintf(ctx->output, "float %s = %gf;\n", instr->result.name, instr->const_value.float_value);
                } else if (instr->result_type.kind == CHIR_TYPE_BOOL) {
                    fprintf(ctx->output, "bool %s = %s;\n", instr->result.name, instr->const_value.bool_value ? "true" : "false");
                } else {
                    fprintf(ctx->output, "int64_t %s = %lld;\n", instr->result.name, (long long)instr->const_value.int_value);
                }
                break;

            case CHIR_OP_ALLOC:
                fprintf(ctx->output, "%s %s;\n",
                    chim_c_codegen_map_type_internal(&instr->alloc_type),
                    instr->result.name);
                break;

            case CHIR_OP_LOAD:
                fprintf(ctx->output, "%s = *(%s)(%s);\n",
                    instr->result.name,
                    chim_c_codegen_map_type_internal(&instr->value_type),
                    instr->addr.name);
                break;

            case CHIR_OP_STORE:
                fprintf(ctx->output, "*((%s)(%s)) = %s;\n",
                    chim_c_codegen_map_type_internal(&instr->value_type),
                    instr->addr.name,
                    instr->value.name);
                break;

            case CHIR_OP_ADD:
                fprintf(ctx->output, "%s = %s + %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_SUB:
                fprintf(ctx->output, "%s = %s - %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_MUL:
                fprintf(ctx->output, "%s = %s * %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_DIV:
                fprintf(ctx->output, "%s = %s / %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_MOD:
                fprintf(ctx->output, "%s = %s %% %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_AND:
                fprintf(ctx->output, "%s = %s && %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_OR:
                fprintf(ctx->output, "%s = %s || %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_XOR:
                fprintf(ctx->output, "%s = %s ^ %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_SHL:
                fprintf(ctx->output, "%s = %s << %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_SHR:
                fprintf(ctx->output, "%s = %s >> %s;\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_EQ:
                fprintf(ctx->output, "%s = (%s == %s);\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_NE:
                fprintf(ctx->output, "%s = (%s != %s);\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_LT:
                fprintf(ctx->output, "%s = (%s < %s);\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_LE:
                fprintf(ctx->output, "%s = (%s <= %s);\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_GT:
                fprintf(ctx->output, "%s = (%s > %s);\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_GE:
                fprintf(ctx->output, "%s = (%s >= %s);\n",
                    instr->result.name, instr->left.name, instr->right.name);
                break;

            case CHIR_OP_RET: {
                const char* value = instr->has_value ? instr->value.name : "0";
                fprintf(ctx->output, "return %s;\n", value);
                break;
            }

            case CHIR_OP_BR:
                fprintf(ctx->output, "goto %s;\n", instr->label.name);
                break;

            case CHIR_OP_BRT:
                fprintf(ctx->output, "if (%s) goto %s;\n", instr->cond.name, instr->true_label.name);
                break;

            case CHIR_OP_BRF:
                fprintf(ctx->output, "if (!%s) goto %s;\n", instr->cond.name, instr->false_label.name);
                break;

            case CHIR_OP_LABEL:
                fprintf(ctx->output, "%s:\n", instr->label.name);
                break;

            case CHIR_OP_CALL: {
                const char* result_type = chim_c_codegen_map_type_internal(&instr->call_return_type);
                if (instr->has_result) {
                    fprintf(ctx->output, "%s %s = %s(",
                        result_type, instr->result.name, instr->callee.name);
                } else {
                    fprintf(ctx->output, "%s(",
                        instr->callee.name);
                }

                bool first_arg = true;
                for (size_t i = 0; i < instr->arg_count; i++) {
                    if (!first_arg) chim_c_codegen_write(ctx, ", ");
                    first_arg = false;
                    chim_c_codegen_write(ctx, instr->args[i].name);
                }
                fprintf(ctx->output, ");\n");
                break;
            }

            case CHIR_OP_NEG:
                fprintf(ctx->output, "%s = -%s;\n", instr->result.name, instr->value.name);
                break;

            case CHIR_OP_NOT:
                fprintf(ctx->output, "%s = !%s;\n", instr->result.name, instr->value.name);
                break;

            case CHIR_OP_ZEXT:
                fprintf(ctx->output, "%s = (%s)(unsigned long long)%s;\n",
                    instr->result.name,
                    chim_c_codegen_map_type_internal(&instr->target_type),
                    instr->value.name);
                break;

            case CHIR_OP_TRUNC:
                fprintf(ctx->output, "%s = (%s)(long long)%s;\n",
                    instr->result.name,
                    chim_c_codegen_map_type_internal(&instr->target_type),
                    instr->value.name);
                break;

            case CHIR_OP_SEXT:
                fprintf(ctx->output, "%s = (%s)(long long)%s;\n",
                    instr->result.name,
                    chim_c_codegen_map_type_internal(&instr->target_type),
                    instr->value.name);
                break;

            case CHIR_OP_FPEXT:
                chim_sprintf("%s = (%s)%s;\n",
                    instr->result.name,
                    chim_c_codegen_map_type_internal(&instr->target_type),
                    instr->value.name);
                chim_c_codegen_write(ctx, result);
                break;

            case CHIR_OP_FPTRUNC:
                fprintf(ctx->output, "%s = (%s)%s;\n",
                    instr->result.name,
                    chim_c_codegen_map_type_internal(&instr->target_type),
                    instr->value.name);
                break;

            case CHIR_OP_ITOF:
                fprintf(ctx->output, "%s = (double)%s;\n", instr->result.name, instr->value.name);
                break;

            case CHIR_OP_FTOI:
                fprintf(ctx->output, "%s = (int64_t)%s;\n", instr->result.name, instr->value.name);
                break;

            default:
                chim_c_codegen_write(ctx, "/* unknown instruction */\n");
                break;
        }

        instr = instr->next;
    }

    ctx->indent_level--;
    chim_c_codegen_write_indented(ctx, "}\n\n");
}

static void chim_c_codegen_emit_module(chim_c_codegen_context_t* ctx, chim_ir_module_t* module) {
    if (!module) return;

    chim_c_codegen_write(ctx, "/* Chim 编译模块: ");
    chim_c_codegen_write(ctx, module->name);
    chim_c_codegen_write(ctx, " */\n\n");

    chim_c_codegen_write(ctx, "#include \"chim_types.h\"\n");
    chim_c_codegen_write(ctx, "#include \"chim_runtime.h\"\n\n");

    chim_ir_function_t* func = module->functions;
    while (func) {
        chim_c_codegen_emit_function(ctx, func);
        func = func->next;
    }
}

chim_codegen_t* chim_c_codegen_create(chim_ir_module_t* module) {
    chim_c_codegen_config_t* config = (chim_c_codegen_config_t*)chim_alloc(sizeof(chim_c_codegen_config_t));
    if (!config) return NULL;

    chim_c_codegen_config_init(config);

    chim_codegen_t* codegen = (chim_codegen_t*)chim_alloc(sizeof(chim_codegen_t));
    if (!codegen) {
        chim_free(config);
        return NULL;
    }

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)chim_alloc(sizeof(chim_c_codegen_context_t));
    if (!ctx) {
        chim_free(codegen);
        chim_free(config);
        return NULL;
    }

    ctx->output = NULL;
    ctx->indent_level = 0;
    ctx->config = config;
    ctx->module = module;
    ctx->temp_counter = 0;
    ctx->label_counter = 0;

    codegen->context = ctx;
    codegen->module = module;

    return codegen;
}

chim_codegen_t* chim_c_codegen_create_with_config(chim_ir_module_t* module,
    const chim_c_codegen_config_t* config) {
    chim_codegen_t* codegen = chim_c_codegen_create(module);
    if (codegen && config) {
        memcpy(codegen->context->config, config, sizeof(chim_c_codegen_config_t));
    }
    return codegen;
}

void chim_c_codegen_destroy(chim_codegen_t* codegen) {
    if (!codegen) return;

    if (codegen->context) {
        if (codegen->context->output) {
            fclose(codegen->context->output);
        }
        chim_free(codegen->context->config);
        chim_free(codegen->context);
    }

    chim_free(codegen);
}

void chim_c_codegen_config_init(chim_c_codegen_config_t* config) {
    if (!config) return;

    config->base.target = CHIM_TARGET_C;
    config->base.optimization_level = 0;
    config->base.emit_debug_info = false;
    config->base.output_filename = NULL;
    config->generate_header = false;
    config->header_filename = NULL;
    config->generate_main_wrapper = true;
    config->use_stdlib = true;
    config->enable_exceptions = false;
    config->c_standard = 11;
}

void chim_c_codegen_config_set_standard(chim_c_codegen_config_t* config, int standard) {
    if (!config) return;
    config->c_standard = standard;
}

void chim_c_codegen_config_enable_header(chim_c_codegen_config_t* config, bool enable,
    const char* filename) {
    if (!config) return;
    config->generate_header = enable;
    if (filename) {
        config->header_filename = chim_strdup(filename);
    }
}

void chim_c_codegen_config_enable_main(chim_c_codegen_config_t* config, bool enable) {
    if (!config) return;
    config->generate_main_wrapper = enable;
}

bool chim_c_codegen_emit_runtime(chim_codegen_t* codegen) {
    if (!codegen || !codegen->context) return false;

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)codegen->context;

    chim_c_codegen_emit_runtime_functions(ctx);

    return true;
}

bool chim_c_codegen_emit_runtime_header(chim_codegen_t* codegen) {
    if (!codegen || !codegen->context) return false;

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)codegen->context;

    chim_c_codegen_emit_type_aliases(ctx);

    return true;
}

const char* chim_c_codegen_map_type(chim_ir_type_t* type) {
    return chim_c_codegen_map_type_internal(type);
}

const char* chim_c_codegen_map_type_name(const char* chim_type) {
    if (!chim_type) return "void";

    if (strcmp(chim_type, "void") == 0) return "void";
    if (strcmp(chim_type, "bool") == 0) return "bool";
    if (strcmp(chim_type, "int") == 0) return "int64_t";
    if (strcmp(chim_type, "float") == 0) return "double";
    if (strcmp(chim_type, "string") == 0) return "chim_string_t";
    if (strcmp(chim_type, "list") == 0) return "chim_list_t";

    return chim_type;
}

void chim_c_codegen_emit_builtin_decls(chim_codegen_t* codegen) {
    if (!codegen || !codegen->context) return;

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)codegen->context;

    chim_c_codegen_write(ctx, "/* 内置函数声明 */\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_int(int64_t value);\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_float(double value);\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_string(chim_string_t* str);\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_bool(bool value);\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_newline(void);\n");
    chim_c_codegen_write_indented(ctx, "chim_string_t chim_string_create(const char* data);\n");
    chim_c_codegen_write_indented(ctx, "void chim_string_destroy(chim_string_t* str);\n");
    chim_c_codegen_write_indented(ctx, "chim_list_t chim_list_create(size_t element_size);\n");
    chim_c_codegen_write_indented(ctx, "void chim_list_push(chim_list_t* list, void* element);\n");
    chim_c_codegen_write_indented(ctx, "void* chim_list_get(chim_list_t* list, size_t index);\n");
    chim_c_codegen_write_indented(ctx, "void chim_list_destroy(chim_list_t* list);\n");
    chim_c_codegen_newline(ctx);
}

bool chim_c_codegen_emit_header(chim_codegen_t* codegen, const char* filename) {
    if (!codegen || !codegen->context) return false;

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)codegen->context;

    FILE* output = fopen(filename, "w");
    if (!output) return false;

    ctx->output = output;

    chim_c_codegen_emit_type_aliases(ctx);

    fclose(output);
    ctx->output = NULL;

    return true;
}

void chim_c_codegen_emit_memory_funcs(chim_codegen_t* codegen) {
    if (!codegen || !codegen->context) return;

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)codegen->context;

    chim_c_codegen_write(ctx, "/* 内存管理函数 */\n");
    chim_c_codegen_write_indented(ctx, "void* chim_alloc(size_t size);\n");
    chim_c_codegen_write_indented(ctx, "void chim_free(void* ptr);\n");
    chim_c_codegen_write_indented(ctx, "void* chim_realloc(void* ptr, size_t size);\n");
    chim_c_codegen_write_indented(ctx, "void chim_memset(void* ptr, int value, size_t size);\n");
    chim_c_codegen_write_indented(ctx, "void chim_memcpy(void* dest, void* src, size_t size);\n");
    chim_c_codegen_write_indented(ctx, "int chim_memcmp(void* a, void* b, size_t size);\n");
    chim_c_codegen_newline(ctx);
}

void chim_c_codegen_emit_list_funcs(chim_codegen_t* codegen) {
    if (!codegen || !codegen->context) return;

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)codegen->context;

    chim_c_codegen_write(ctx, "/* 列表操作函数声明 */\n");
    chim_c_codegen_write_indented(ctx, "chim_list_t chim_list_create(size_t element_size);\n");
    chim_c_codegen_write_indented(ctx, "void chim_list_push(chim_list_t* list, void* element);\n");
    chim_c_codegen_write_indented(ctx, "void* chim_list_get(chim_list_t* list, size_t index);\n");
    chim_c_codegen_write_indented(ctx, "size_t chim_list_length(chim_list_t* list);\n");
    chim_c_codegen_write_indented(ctx, "void chim_list_destroy(chim_list_t* list);\n");
    chim_c_codegen_newline(ctx);
}

void chim_c_codegen_emit_string_funcs(chim_codegen_t* codegen) {
    if (!codegen || !codegen->context) return;

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)codegen->context;

    chim_c_codegen_write(ctx, "/* 字符串操作函数声明 */\n");
    chim_c_codegen_write_indented(ctx, "chim_string_t chim_string_create(const char* data);\n");
    chim_c_codegen_write_indented(ctx, "chim_string_t chim_string_from_literal(const char* data);\n");
    chim_c_codegen_write_indented(ctx, "void chim_string_destroy(chim_string_t* str);\n");
    chim_c_codegen_write_indented(ctx, "chim_string_t chim_string_concat(chim_string_t* a, chim_string_t* b);\n");
    chim_c_codegen_write_indented(ctx, "size_t chim_string_length(chim_string_t* str);\n");
    chim_c_codegen_write_indented(ctx, "const char* chim_string_cstr(chim_string_t* str);\n");
    chim_c_codegen_newline(ctx);
}

void chim_c_codegen_emit_print_funcs(chim_codegen_t* codegen) {
    if (!codegen || !codegen->context) return;

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)codegen->context;

    chim_c_codegen_write(ctx, "/* 打印函数声明 */\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_int(int64_t value);\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_float(double value);\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_string(chim_string_t* str);\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_bool(bool value);\n");
    chim_c_codegen_write_indented(ctx, "void chim_print_newline(void);\n");
    chim_c_codegen_newline(ctx);
}

bool chim_c_codegen_emit_to_file(chim_codegen_t* codegen, const char* filename) {
    if (!codegen || !codegen->context) return false;

    chim_c_codegen_context_t* ctx = (chim_c_codegen_context_t*)codegen->context;

    FILE* output = fopen(filename, "w");
    if (!output) return false;

    ctx->output = output;

    chim_c_codegen_write(output, "/* Generated by Chim 3.1 Compiler */\n");
    chim_c_codegen_write(output, "/* Target: C (TCC compatible) */\n\n");

    chim_c_codegen_write(output, "#include \"chim_types.h\"\n");
    chim_c_codegen_write(output, "#include \"chim_runtime.h\"\n");
    chim_c_codegen_write(output, "#include <stdio.h>\n");
    chim_c_codegen_write(output, "#include <stdlib.h>\n");
    chim_c_codegen_write(output, "#include <string.h>\n\n");

    chim_c_codegen_emit_builtin_decls(codegen);

    chim_c_codegen_emit_module(ctx, codegen->module);

    if (ctx->config->generate_main_wrapper) {
        chim_c_codegen_write(output, "/* 主函数入口 */\n");
        chim_c_codegen_write_indented(output, "int main(int argc, char** argv) {\n");
        chim_c_codegen_write_indented(output, "    /* TODO: 添加命令行参数处理 */\n");
        chim_c_codegen_write_indented(output, "    return 0;\n");
        chim_c_codegen_write_indented(output, "}\n");
    }

    fclose(output);
    ctx->output = NULL;

    return true;
}
