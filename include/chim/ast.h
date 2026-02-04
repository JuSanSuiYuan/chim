/**
 * @file ast.h
 * @brief Chim 3.1 抽象语法树定义
 */

#ifndef CHIM_AST_H
#define CHIM_AST_H

#include "token.h"
#include "common.h"

/* AST 节点类型枚举 */
typedef enum {
    CHIM_AST_PROGRAM,
    CHIM_AST_FUNCTION,
    CHIM_AST_VARIABLE_DECL,
    CHIM_AST_CONST_DECL,
    CHIM_AST_PARAMETER,
    CHIM_AST_BLOCK,
    CHIM_AST_EXPR_STMT,
    CHIM_AST_IF_EXPR,
    CHIM_AST_IF_STMT,
    CHIM_AST_FOR_STMT,
    CHIM_AST_WHILE_STMT,
    CHIM_AST_MATCH_EXPR,
    CHIM_AST_MATCH_CASE,
    CHIM_AST_RETURN_STMT,
    CHIM_AST_BREAK_STMT,
    CHIM_AST_CONTINUE_STMT,
    CHIM_AST_ASSIGNMENT,
    CHIM_AST_BINARY_OP,
    CHIM_AST_UNARY_OP,
    CHIM_AST_CALL,
    CHIM_AST_IDENTIFIER,
    CHIM_AST_INT_LITERAL,
    CHIM_AST_FLOAT_LITERAL,
    CHIM_AST_STRING_LITERAL,
    CHIM_AST_CHAR_LITERAL,
    CHIM_AST_BOOL_LITERAL,
    CHIM_AST_NIL_LITERAL,
    CHIM_AST_LIST_LITERAL,
    CHIM_AST_TUPLE_LITERAL,
    CHIM_AST_RECORD_LITERAL,
    CHIM_AST_RECORD_FIELD,
    CHIM_AST_INDEX_ACCESS,
    CHIM_AST_FIELD_ACCESS,
    CHIM_AST_TYPE_REF,
    CHIM_AST_FUNC_TYPE,
    CHIM_AST_LIST_TYPE,
    CHIM_AST_TUPLE_TYPE,
} chim_ast_node_type_t;

/* 前向声明 */
typedef struct chim_ast_node_t chim_ast_node_t;
typedef struct chim_ast_symbol_t chim_ast_symbol_t;

/* AST 节点公共部分 */
typedef struct {
    chim_ast_node_type_t type;
    chim_location_t location;
    struct chim_ast_node_t* parent;
    struct chim_ast_symbol_t* symbol;
} chim_ast_base_t;

/* 表达式类型 */
typedef enum {
    CHIM_EXPR_SIMPLE,
    CHIM_EXPR_BINARY,
    CHIM_EXPR_UNARY,
    CHIM_EXPR_CALL,
    CHIM_EXPR_SUBSCRIPT,
} chim_ast_expr_type_t;

/* 符号类型 */
typedef enum {
    CHIM_SYMBOL_VARIABLE,
    CHIM_SYMBOL_FUNCTION,
    CHIM_SYMBOL_PARAMETER,
    CHIM_SYMBOL_TYPE,
} chim_ast_symbol_type_t;

/* 符号表 */
typedef struct chim_ast_symbol_table_t {
    chim_ast_symbol_t** symbols;
    size_t count;
    size_t capacity;
    struct chim_ast_symbol_table_t* parent;
} chim_ast_symbol_table_t;

/* 符号定义 */
struct chim_ast_symbol_t {
    chim_ast_symbol_type_t type;
    const char* name;
    chim_ast_node_t* declaration;
    chim_ast_type_t* value_type;
    bool is_mutable;
    bool is_used;
    bool is_assigned;
};

/* 类型定义 */
typedef struct chim_ast_type_t {
    const char* name;
    enum {
        CHIM_TYPE_PRIMITIVE,
        CHIM_TYPE_LIST,
        CHIM_TYPE_TUPLE,
        CHIM_TYPE_FUNC,
        CHIM_TYPE_USER,
    } kind;
    struct chim_ast_type_t* elem_type;
    struct chim_ast_type_t* param_type;
    struct chim_ast_type_t* return_type;
} chim_ast_type_t;

/* 程序节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t** functions;
    size_t num_functions;
    chim_ast_node_t** declarations;
    size_t num_declarations;
    chim_ast_symbol_table_t* global_symbols;
} chim_ast_program_t;

/* 函数定义节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* name;
    chim_ast_node_t** params;
    size_t num_params;
    chim_ast_node_t* return_type;
    chim_ast_node_t* body;
    chim_ast_type_t* func_type;
} chim_ast_function_t;

/* 变量声明节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* name;
    chim_ast_node_t* var_type;
    chim_ast_node_t* initializer;
    bool is_mutable;
} chim_ast_variable_decl_t;

/* 常量声明节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* name;
    chim_ast_node_t* const_type;
    chim_ast_node_t* value;
} chim_ast_const_decl_t;

/* 参数节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* name;
    chim_ast_node_t* param_type;
} chim_ast_parameter_t;

/* 代码块节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t** statements;
    size_t num_statements;
    chim_ast_symbol_table_t* local_symbols;
} chim_ast_block_t;

/* 表达式语句节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* expression;
} chim_ast_expr_stmt_t;

/* if 表达式节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* condition;
    chim_ast_node_t* then_branch;
    chim_ast_node_t* else_branch;
} chim_ast_if_expr_t;

/* if 语句节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* condition;
    chim_ast_node_t* then_branch;
    chim_ast_node_t** elif_branches;
    size_t num_elifs;
    chim_ast_node_t* else_branch;
} chim_ast_if_stmt_t;

/* for 循环节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* iterator;
    chim_ast_node_t* iterable;
    chim_ast_node_t* body;
} chim_ast_for_stmt_t;

/* while 循环节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* condition;
    chim_ast_node_t* body;
} chim_ast_while_stmt_t;

/* match 表达式节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* matched_expr;
    chim_ast_node_t** cases;
    size_t num_cases;
} chim_ast_match_expr_t;

/* match case 节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* pattern;
    chim_ast_node_t* body;
} chim_ast_match_case_t;

/* return 语句节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* return_value;
} chim_ast_return_stmt_t;

/* break 语句节点 */
typedef struct {
    chim_ast_base_t base;
} chim_ast_break_stmt_t;

/* continue 语句节点 */
typedef struct {
    chim_ast_base_t base;
} chim_ast_continue_stmt_t;

/* 赋值表达式节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* target;
    chim_ast_node_t* value;
} chim_ast_assignment_t;

/* 二元运算表达式节点 */
typedef struct {
    chim_ast_base_t base;
    chim_token_type_t op;
    chim_ast_node_t* left;
    chim_ast_node_t* right;
} chim_ast_binary_op_t;

/* 一元运算表达式节点 */
typedef struct {
    chim_ast_base_t base;
    chim_token_type_t op;
    chim_ast_node_t* operand;
} chim_ast_unary_op_t;

/* 函数调用表达式节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* function;
    chim_ast_node_t** arguments;
    size_t num_arguments;
} chim_ast_call_t;

/* 标识符表达式节点 */
typedef struct {
    chim_ast_base_t base;
    const char* name;
    bool is_lvalue;
} chim_ast_identifier_t;

/* 整数字面量节点 */
typedef struct {
    chim_ast_base_t base;
    int64_t value;
} chim_ast_int_literal_t;

/* 浮点数字面量节点 */
typedef struct {
    chim_ast_base_t base;
    double value;
} chim_ast_float_literal_t;

/* 字符串字面量节点 */
typedef struct {
    chim_ast_base_t base;
    char* value;
} chim_ast_string_literal_t;

/* 字符字面量节点 */
typedef struct {
    chim_ast_base_t base;
    char value;
} chim_ast_char_literal_t;

/* 布尔字面量节点 */
typedef struct {
    chim_ast_base_t base;
    bool value;
} chim_ast_bool_literal_t;

/* nil 字面量节点 */
typedef struct {
    chim_ast_base_t base;
} chim_ast_nil_literal_t;

/* 列表字面量节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t** elements;
    size_t num_elements;
    chim_ast_type_t* elem_type;
} chim_ast_list_literal_t;

/* 元组字面量节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t** elements;
    size_t num_elements;
} chim_ast_tuple_literal_t;

/* 记录字面量节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t** fields;
    size_t num_fields;
} chim_ast_record_literal_t;

/* 记录字段节点 */
typedef struct {
    chim_ast_base_t base;
    const char* name;
    chim_ast_node_t* value;
} chim_ast_record_field_t;

/* 下标访问节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* array;
    chim_ast_node_t* index;
} chim_ast_index_access_t;

/* 字段访问节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* record;
    const char* field_name;
} chim_ast_field_access_t;

/* 类型引用节点 */
typedef struct {
    chim_ast_base_t base;
    const char* type_name;
} chim_ast_type_ref_t;

/* 函数类型节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* param_type;
    chim_ast_node_t* return_type;
} chim_ast_func_type_t;

/* 列表类型节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t* elem_type;
} chim_ast_list_type_t;

/* 元组类型节点 */
typedef struct {
    chim_ast_base_t base;
    chim_ast_node_t** elem_types;
    size_t num_elem_types;
} chim_ast_tuple_type_t;

/* AST 节点联合体 */
struct chim_ast_node_t {
    chim_ast_base_t base;
    union {
        chim_ast_program_t program;
        chim_ast_function_t function;
        chim_ast_variable_decl_t variable_decl;
        chim_ast_const_decl_t const_decl;
        chim_ast_parameter_t parameter;
        chim_ast_block_t block;
        chim_ast_expr_stmt_t expr_stmt;
        chim_ast_if_expr_t if_expr;
        chim_ast_if_stmt_t if_stmt;
        chim_ast_for_stmt_t for_stmt;
        chim_ast_while_stmt_t while_stmt;
        chim_ast_match_expr_t match_expr;
        chim_ast_match_case_t match_case;
        chim_ast_return_stmt_t return_stmt;
        chim_ast_break_stmt_t break_stmt;
        chim_ast_continue_stmt_t continue_stmt;
        chim_ast_assignment_t assignment;
        chim_ast_binary_op_t binary_op;
        chim_ast_unary_op_t unary_op;
        chim_ast_call_t call;
        chim_ast_identifier_t identifier;
        chim_ast_int_literal_t int_literal;
        chim_ast_float_literal_t float_literal;
        chim_ast_string_literal_t string_literal;
        chim_ast_char_literal_t char_literal;
        chim_ast_bool_literal_t bool_literal;
        chim_ast_nil_literal_t nil_literal;
        chim_ast_list_literal_t list_literal;
        chim_ast_tuple_literal_t tuple_literal;
        chim_ast_record_literal_t record_literal;
        chim_ast_record_field_t record_field;
        chim_ast_index_access_t index_access;
        chim_ast_field_access_t field_access;
        chim_ast_type_ref_t type_ref;
        chim_ast_func_type_t func_type;
        chim_ast_list_type_t list_type;
        chim_ast_tuple_type_t tuple_type;
    };
};

/* AST 构建器 */
typedef struct {
    chim_diagnostics_t* diagnostics;
    chim_ast_node_t* current_function;
    chim_ast_symbol_table_t* current_scope;
    size_t error_count;
} chim_ast_builder_t;

/* AST 模块初始化 */
int chim_ast_init(void);
void chim_ast_cleanup(void);

/* AST 节点创建 */
chim_ast_node_t* chim_ast_create_node(chim_ast_node_type_t type, const chim_location_t* location);
void chim_ast_destroy_node(chim_ast_node_t* node);

/* 程序节点操作 */
chim_ast_node_t* chim_ast_create_program(const chim_location_t* location);
void chim_ast_program_add_function(chim_ast_node_t* program, chim_ast_node_t* function);
void chim_ast_program_add_declaration(chim_ast_node_t* program, chim_ast_node_t* decl);

/* 函数节点操作 */
chim_ast_node_t* chim_ast_create_function(
    const chim_location_t* location,
    const char* name,
    chim_ast_node_t** params,
    size_t num_params,
    chim_ast_node_t* return_type,
    chim_ast_node_t* body
);

/* 变量/常量声明操作 */
chim_ast_node_t* chim_ast_create_variable_decl(
    const chim_location_t* location,
    const char* name,
    chim_ast_node_t* var_type,
    chim_ast_node_t* initializer,
    bool is_mutable
);

chim_ast_node_t* chim_ast_create_const_decl(
    const chim_location_t* location,
    const char* name,
    chim_ast_node_t* const_type,
    chim_ast_node_t* value
);

/* 表达式创建 */
chim_ast_node_t* chim_ast_create_identifier(const chim_location_t* location, const char* name);
chim_ast_node_t* chim_ast_create_int_literal(const chim_location_t* location, int64_t value);
chim_ast_node_t* chim_ast_create_float_literal(const chim_location_t* location, double value);
chim_ast_node_t* chim_ast_create_string_literal(const chim_location_t* location, char* value);
chim_ast_node_t* chim_ast_create_char_literal(const chim_location_t* location, char value);
chim_ast_node_t* chim_ast_create_bool_literal(const chim_location_t* location, bool value);
chim_ast_node_t* chim_ast_create_nil_literal(const chim_location_t* location);

/* 列表/元组/记录 */
chim_ast_node_t* chim_ast_create_list_literal(const chim_location_t* location,
    chim_ast_node_t** elements, size_t num_elements);
chim_ast_node_t* chim_ast_create_tuple_literal(const chim_location_t* location,
    chim_ast_node_t** elements, size_t num_elements);
chim_ast_node_t* chim_ast_create_record_literal(const chim_location_t* location,
    chim_ast_node_t** fields, size_t num_fields);
chim_ast_node_t* chim_ast_create_record_field(const chim_location_t* location,
    const char* name, chim_ast_node_t* value);

/* 运算符表达式 */
chim_ast_node_t* chim_ast_create_binary_op(const chim_location_t* location,
    chim_token_type_t op, chim_ast_node_t* left, chim_ast_node_t* right);
chim_ast_node_t* chim_ast_create_unary_op(const chim_location_t* location,
    chim_token_type_t op, chim_ast_node_t* operand);

/* 调用表达式 */
chim_ast_node_t* chim_ast_create_call(const chim_location_t* location,
    chim_ast_node_t* function, chim_ast_node_t** arguments, size_t num_args);

/* 下标/字段访问 */
chim_ast_node_t* chim_ast_create_index_access(const chim_location_t* location,
    chim_ast_node_t* array, chim_ast_node_t* index);
chim_ast_node_t* chim_ast_create_field_access(const chim_location_t* location,
    chim_ast_node_t* record, const char* field_name);

/* 语句创建 */
chim_ast_node_t* chim_ast_create_block(const chim_location_t* location,
    chim_ast_node_t** statements, size_t num_statements);
chim_ast_node_t* chim_ast_create_expr_stmt(const chim_location_t* location,
    chim_ast_node_t* expression);
chim_ast_node_t* chim_ast_create_assignment(const chim_location_t* location,
    chim_ast_node_t* target, chim_ast_node_t* value);

/* 控制流语句 */
chim_ast_node_t* chim_ast_create_if_expr(const chim_location_t* location,
    chim_ast_node_t* condition, chim_ast_node_t* then_branch, chim_ast_node_t* else_branch);
chim_ast_node_t* chim_ast_create_if_stmt(const chim_location_t* location,
    chim_ast_node_t* condition, chim_ast_node_t* then_branch,
    chim_ast_node_t** elif_branches, size_t num_elifs,
    chim_ast_node_t* else_branch);
chim_ast_node_t* chim_ast_create_for_stmt(const chim_location_t* location,
    const char* iterator, chim_ast_node_t* iterable, chim_ast_node_t* body);
chim_ast_node_t* chim_ast_create_while_stmt(const chim_location_t* location,
    chim_ast_node_t* condition, chim_ast_node_t* body);
chim_ast_node_t* chim_ast_create_match_expr(const chim_location_t* location,
    chim_ast_node_t* matched_expr, chim_ast_node_t** cases, size_t num_cases);
chim_ast_node_t* chim_ast_create_match_case(const chim_location_t* location,
    chim_ast_node_t* pattern, chim_ast_node_t* body);
chim_ast_node_t* chim_ast_create_return_stmt(const chim_location_t* location,
    chim_ast_node_t* return_value);
chim_ast_node_t* chim_ast_create_break_stmt(const chim_location_t* location);
chim_ast_node_t* chim_ast_create_continue_stmt(const chim_location_t* location);

/* 类型节点 */
chim_ast_node_t* chim_ast_create_type_ref(const chim_location_t* location, const char* type_name);
chim_ast_node_t* chim_ast_create_func_type(const chim_location_t* location,
    chim_ast_node_t* param_type, chim_ast_node_t* return_type);
chim_ast_node_t* chim_ast_create_list_type(const chim_location_t* location,
    chim_ast_node_t* elem_type);

/* 符号表操作 */
chim_ast_symbol_table_t* chim_ast_symbol_table_create(chim_ast_symbol_table_t* parent);
void chim_ast_symbol_table_destroy(chim_ast_symbol_table_t* table);
bool chim_ast_symbol_table_insert(chim_ast_symbol_table_t* table, chim_ast_symbol_t* symbol);
chim_ast_symbol_t* chim_ast_symbol_table_lookup(chim_ast_symbol_table_t* table, const char* name);

/* AST 打印 */
void chim_ast_print(chim_ast_node_t* node, FILE* output, int indent);
void chim_ast_print_program(chim_ast_node_t* program, FILE* output);

/* AST 验证 */
int chim_ast_validate(chim_ast_node_t* node, chim_diagnostics_t* diagnostics);

/* AST 遍历 */
typedef void (*chim_ast_visit_func_t)(chim_ast_node_t* node, void* user_data);
void chim_ast_visit(chim_ast_node_t* node, chim_ast_visit_func_t pre_visit,
    chim_ast_visit_func_t post_visit, void* user_data);

/* 位置设置 */
void chim_ast_node_set_location(chim_ast_node_t* node, const chim_location_t* location);

#endif /* CHIM_AST_H */
