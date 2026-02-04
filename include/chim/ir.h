/**
 * @file ir.h
 * @brief Chim 3.1 中间表示 (IR) 定义
 *
 * 使用三地址码 (Three Address Code) 作为中间表示
 */

#ifndef CHIM_IR_H
#define CHIM_IR_H

#include "common.h"
#include "ast.h"

/* IR 操作码枚举 */
typedef enum {
    /* 加载/存储 */
    CHIM_IR_LOAD,
    CHIM_IR_STORE,
    CHIM_IR_ALLOCA,

    /* 算术运算 */
    CHIM_IR_ADD,
    CHIM_IR_SUB,
    CHIM_IR_MUL,
    CHIM_IR_DIV,
    CHIM_IR_MOD,

    /* 比较运算 */
    CHIM_IR_EQ,
    CHIM_IR_NE,
    CHIM_IR_LT,
    CHIM_IR_LE,
    CHIM_IR_GT,
    CHIM_IR_GE,

    /* 逻辑运算 */
    CHIM_IR_AND,
    CHIM_IR_OR,
    CHIM_IR_NOT,

    /* 位运算 */
    CHIM_IR_BITAND,
    CHIM_IR_BITOR,
    CHIM_IR_BITXOR,
    CHIM_IR_LSHIFT,
    CHIM_IR_RSHIFT,

    /* 函数调用 */
    CHIM_IR_CALL,
    CHIM_IR_CALL_INDIRECT,

    /* 内存操作 */
    CHIM_IR_GETELEMENTPTR,
    CHIM_IR_LOAD_ELEMENT,
    CHIM_IR_STORE_ELEMENT,

    /* 类型转换 */
    CHIM_IR_ZEXT,      /* 零扩展 */
    CHIM_IR_SEXT,      /* 符号扩展 */
    CHIM_IR_TRUNC,     /* 截断 */
    CHIM_IR_FP_TO_INT,
    CHIM_IR_INT_TO_FP,

    /* 控制流 */
    CHIM_IR_BR,
    CHIM_IR_COND_BR,
    CHIM_IR_RET,
    CHIM_IR_LABEL,

    /* PHI 节点 (SSA) */
    CHIM_IR_PHI,

    /* 其他 */
    CHIM_IR_MOVE,
    CHIM_IR_NOP,
} chim_ir_opcode_t;

/* IR 值类型 */
typedef enum {
    CHIM_IR_VALUE_INVALID,
    CHIM_IR_VALUE_CONSTANT,
    CHIM_IR_VALUE_REGISTER,
    CHIM_IR_VALUE_TEMP,
    CHIM_IR_VALUE_GLOBAL,
    CHIM_IR_VALUE_LABEL,
} chim_ir_value_type_t;

/* IR 值 */
typedef struct chim_ir_value_t {
    chim_ir_value_type_t type;
    const char* name;
    struct chim_ir_type_t* value_type;
    union {
        int64_t int_value;
        double float_value;
        char* string_value;
    };
} chim_ir_value_t;

/* IR 类型 */
typedef struct chim_ir_type_t {
    enum {
        CHIM_IR_TYPE_VOID,
        CHIM_IR_TYPE_INT,
        CHIM_IR_TYPE_FLOAT,
        CHIM_IR_TYPE_BOOL,
        CHIM_IR_TYPE_STRING,
        CHIM_IR_TYPE_POINTER,
        CHIM_IR_TYPE_ARRAY,
        CHIM_IR_TYPE_STRUCT,
        CHIM_IR_TYPE_FUNCTION,
    } kind;
    size_t size;           /* 字节大小 */
    size_t alignment;      /* 对齐要求 */
    struct chim_ir_type_t* elem_type;    /* 元素类型 (用于指针、数组) */
    struct chim_ir_type_t* return_type;  /* 返回类型 (用于函数) */
    chim_ast_type_t* source_type;        /* 源 AST 类型 */
} chim_ir_type_t;

/* IR 基本块 */
typedef struct chim_ir_basic_block_t {
    const char* name;
    struct chim_ir_instr_t* first_instr;
    struct chim_ir_instr_t* last_instr;
    struct chim_ir_basic_block_t* next;
    int num_predecessors;
    struct chim_ir_basic_block_t** predecessors;
    int num_successors;
    struct chim_ir_basic_block_t** successors;
    struct chim_ir_function_t* parent;
} chim_ir_basic_block_t;

/* IR 指令 */
typedef struct chim_ir_instr_t {
    chim_ir_opcode_t opcode;
    struct chim_ir_value_t* result;
    struct chim_ir_value_t** operands;
    size_t num_operands;
    struct chim_ir_basic_block_t* target_block;
    struct chim_ir_instr_t* next;
    struct chim_ir_basic_block_t* parent;
    bool is_terminator;
} chim_ir_instr_t;

/* IR 函数 */
typedef struct chim_ir_function_t {
    const char* name;
    chim_ir_type_t* return_type;
    chim_ir_type_t** param_types;
    size_t num_params;
    struct chim_ir_basic_block_t* entry_block;
    struct chim_ir_basic_block_t* exit_block;
    struct chim_ir_basic_block_t* first_block;
    struct chim_ir_basic_block_t* last_block;
    struct chim_ir_value_t** local_vars;
    size_t num_local_vars;
    struct chim_ir_function_t* next;
    struct chim_ir_module_t* parent;
    bool is_extern;
} chim_ir_function_t;

/* IR 全局变量 */
typedef struct chim_ir_global_var_t {
    const char* name;
    chim_ir_type_t* type;
    struct chim_ir_value_t* initializer;
    bool is_constant;
    struct chim_ir_global_var_t* next;
    struct chim_ir_module_t* parent;
} chim_ir_global_var_t;

/* IR 模块 */
typedef struct chim_ir_module_t {
    const char* name;
    struct chim_ir_global_var_t* globals;
    struct chim_ir_function_t* functions;
    struct chim_ir_type_t** types;
    size_t num_types;
    size_t next_temp_id;
    size_t next_label_id;
} chim_ir_module_t;

/* IR 构建器 */
typedef struct {
    chim_ir_module_t* module;
    chim_ir_function_t* current_function;
    chim_ir_basic_block_t* current_block;
    chim_ast_symbol_table_t* symbol_table;
    size_t error_count;
} chim_ir_builder_t;

/* IR 模块操作 */
chim_ir_module_t* chim_ir_module_create(const char* name);
void chim_ir_module_destroy(chim_ir_module_t* module);

/* IR 类型操作 */
chim_ir_type_t* chim_ir_type_create_void(void);
chim_ir_type_t* chim_ir_type_create_int(int width);
chim_ir_type_t* chim_ir_type_create_float(void);
chim_ir_type_t* chim_ir_type_create_bool(void);
chim_ir_type_t* chim_ir_type_create_string(void);
chim_ir_type_t* chim_ir_type_create_pointer(chim_ir_type_t* elem_type);
chim_ir_type_t* chim_ir_type_create_array(chim_ir_type_t* elem_type, size_t num_elements);
chim_ir_type_t* chim_ir_type_create_function(chim_ir_type_t* return_type,
    chim_ir_type_t** param_types, size_t num_params);

/* IR 值操作 */
chim_ir_value_t* chim_ir_value_create(chim_ir_value_type_t type, const char* name, chim_ir_type_t* value_type);
chim_ir_value_t* chim_ir_value_create_const_int(int64_t value);
chim_ir_value_t* chim_ir_value_create_const_float(double value);
chim_ir_value_t* chim_ir_value_create_const_string(const char* value);
chim_ir_value_t* chim_ir_value_create_temp(chim_ir_builder_t* builder, chim_ir_type_t* type);
chim_ir_value_t* chim_ir_value_create_label(chim_ir_builder_t* builder);
void chim_ir_value_destroy(chim_ir_value_t* value);

/* IR 指令创建 */
chim_ir_instr_t* chim_ir_instr_create(chim_ir_opcode_t opcode);
void chim_ir_instr_set_result(chim_ir_instr_t* instr, chim_ir_value_t* result);
void chim_ir_instr_add_operand(chim_ir_instr_t* instr, chim_ir_value_t* operand);
void chim_ir_instr_destroy(chim_ir_instr_t* instr);

/* IR 基本块操作 */
chim_ir_basic_block_t* chim_ir_basic_block_create(chim_ir_function_t* func, const char* name);
void chim_ir_basic_block_append_instr(chim_ir_basic_block_t* block, chim_ir_instr_t* instr);
void chim_ir_basic_block_destroy(chim_ir_basic_block_t* block);

/* IR 函数操作 */
chim_ir_function_t* chim_ir_function_create(chim_ir_module_t* module, const char* name,
    chim_ir_type_t* return_type, bool is_extern);
void chim_ir_function_append_block(chim_ir_function_t* func, chim_ir_basic_block_t* block);
void chim_ir_function_destroy(chim_ir_function_t* func);

/* IR 全局变量操作 */
chim_ir_global_var_t* chim_ir_global_var_create(chim_ir_module_t* module,
    const char* name, chim_ir_type_t* type, bool is_constant);
void chim_ir_global_var_set_initializer(chim_ir_global_var_t* global, chim_ir_value_t* value);
void chim_ir_global_var_destroy(chim_ir_global_var_t* global);

/* IR 构建器操作 */
chim_ir_builder_t* chim_ir_builder_create(chim_ir_module_t* module);
void chim_ir_builder_destroy(chim_ir_builder_t* builder);
void chim_ir_builder_set_function(chim_ir_builder_t* builder, chim_ir_function_t* func);
void chim_ir_builder_set_block(chim_ir_builder_t* builder, chim_ir_basic_block_t* block);

/* 从 AST 构建 IR */
chim_ir_module_t* chim_ir_build_from_ast(chim_ast_node_t* ast, chim_diagnostics_t* diag);

/* 函数声明 */
chim_ir_function_t* chim_ir_build_function_decl(chim_ir_builder_t* builder, chim_ast_node_t* func_node);

/* 函数体 */
void chim_ir_build_function_body(chim_ir_builder_t* builder, chim_ir_function_t* func,
    chim_ast_node_t* func_node);

/* 表达式编译 */
chim_ir_value_t* chim_ir_build_expr(chim_ir_builder_t* builder, chim_ast_node_t* expr);

/* 语句编译 */
void chim_ir_build_stmt(chim_ir_builder_t* builder, chim_ast_node_t* stmt);

/* 指令发射 */
chim_ir_instr_t* chim_ir_build_alloc(chim_ir_builder_t* builder, const char* name,
    chim_ir_type_t* type);
chim_ir_instr_t* chim_ir_build_load(chim_ir_builder_t* builder, chim_ir_value_t* src);
chim_ir_instr_t* chim_ir_build_store(chim_ir_builder_t* builder, chim_ir_value_t* dst,
    chim_ir_value_t* value);
chim_ir_instr_t* chim_ir_build_binary_op(chim_ir_builder_t* builder, chim_ir_opcode_t op,
    chim_ir_value_t* left, chim_ir_value_t* right);
chim_ir_instr_t* chim_ir_build_call(chim_ir_builder_t* builder, const char* func_name,
    chim_ir_value_t** args, size_t num_args);
chim_ir_instr_t* chim_ir_build_ret(chim_ir_builder_t* builder, chim_ir_value_t* value);
chim_ir_instr_t* chim_ir_build_br(chim_ir_builder_t* builder, chim_ir_basic_block_t* target);
chim_ir_instr_t* chim_ir_build_cond_br(chim_ir_builder_t* builder, chim_ir_value_t* condition,
    chim_ir_basic_block_t* true_target, chim_ir_basic_block_t* false_target);
chim_ir_instr_t* chim_ir_build_label(chim_ir_builder_t* builder, chim_ir_value_t* label);

/* 打印 IR */
void chim_ir_print_module(chim_ir_module_t* module, FILE* output);
void chim_ir_print_function(chim_ir_function_t* func, FILE* output);
void chim_ir_print_basic_block(chim_ir_basic_block_t* block, FILE* output);
void chim_ir_print_instruction(chim_ir_instr_t* instr, FILE* output);

/* IR 验证 */
bool chim_ir_verify_module(chim_ir_module_t* module, char** error_msg);
bool chim_ir_verify_function(chim_ir_function_t* func, char** error_msg);

/* SSA 转换 */
void chim_ir_to_ssa(chim_ir_module_t* module);
void chim_ir_from_ssa(chim_ir_module_t* module);

#endif /* CHIM_IR_H */
