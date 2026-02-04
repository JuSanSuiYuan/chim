/**
 * @file parser.h
 * @brief Chim 3.1 语法解析器头文件
 */

#ifndef CHIM_PARSER_H
#define CHIM_PARSER_H

#include "token.h"
#include "ast.h"

/* 解析器上下文 */
typedef struct {
    chim_token_array_t* tokens;
    size_t current;
    chim_diagnostics_t* diagnostics;
    chim_ast_builder_t builder;
    bool error_recovery;
} chim_parser_t;

/* 创建解析器 */
chim_parser_t* chim_parser_create(chim_token_array_t* tokens, chim_diagnostics_t* diag);
void chim_parser_destroy(chim_parser_t* parser);

/* 解析入口 */
chim_ast_node_t* chim_parser_parse(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_expression(chim_parser_t* parser);

/* 解析函数 */
chim_ast_node_t* chim_parser_parse_program(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_function(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_variable_decl(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_const_decl(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_parameters(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_block(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_statement(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_if_stmt(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_for_stmt(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_while_stmt(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_match_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_return_stmt(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_break_stmt(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_continue_stmt(chim_parser_t* parser);

/* 表达式解析 */
chim_ast_node_t* chim_parser_parse_assignment_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_or_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_and_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_bitwise_or_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_bitwise_xor_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_bitwise_and_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_equality_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_relational_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_shift_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_additive_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_multiplicative_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_unary_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_postfix_expr(chim_parser_t* parser);
chim_ast_node_t* chim_parser_parse_primary_expr(chim_parser_t* parser);

/* 模式解析 */
chim_ast_node_t* chim_parser_parse_pattern(chim_parser_t* parser);

/* 类型解析 */
chim_ast_node_t* chim_parser_parse_type(chim_parser_t* parser);

/* 工具函数 */
chim_token_t* chim_parser_current(chim_parser_t* parser);
chim_token_t* chim_parser_peek(chim_parser_t* parser, size_t offset);
chim_token_t* chim_parser_advance(chim_parser_t* parser);
bool chim_parser_match(chim_parser_t* parser, chim_token_type_t type);
bool chim_parser_match_any(chim_parser_t* parser, const chim_token_type_t* types, size_t count);
void chim_parser_expect(chim_parser_t* parser, chim_token_type_t type, const char* expected_name);
bool chim_parser_at_end(chim_parser_t* parser);

/* 错误恢复 */
void chim_parser_synchronize(chim_parser_t* parser);
bool chim_parser_recover_to(chim_parser_t* parser, const chim_token_type_t* sync_tokens, size_t count);

#endif /* CHIM_PARSER_H */
