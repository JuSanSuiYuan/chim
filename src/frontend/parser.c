/**
 * @file parser.c
 * @brief Chim 3.1 语法解析器实现
 */

#include <stdio.h>
#include <stdlib.h>

#include "parser.h"
#include "lexer.h"

/* 创建解析器 */
chim_parser_t* chim_parser_create(chim_token_array_t* tokens, chim_diagnostics_t* diag) {
    chim_parser_t* parser = (chim_parser_t*)chim_alloc(sizeof(chim_parser_t));
    if (!parser) return NULL;

    parser->tokens = tokens;
    parser->current = 0;
    parser->diagnostics = diag;
    parser->error_recovery = false;

    return parser;
}

/* 销毁解析器 */
void chim_parser_destroy(chim_parser_t* parser) {
    if (!parser) return;
    chim_free(parser);
}

/* 获取当前 Token */
chim_token_t* chim_parser_current(chim_parser_t* parser) {
    if (parser->current < parser->tokens->count) {
        return parser->tokens->tokens[parser->current];
    }
    return NULL;
}

/* 预查看 Token */
chim_token_t* chim_parser_peek(chim_parser_t* parser, size_t offset) {
    size_t index = parser->current + offset;
    if (index < parser->tokens->count) {
        return parser->tokens->tokens[index];
    }
    return NULL;
}

/* 前进一个 Token */
chim_token_t* chim_parser_advance(chim_parser_t* parser) {
    if (parser->current < parser->tokens->count) {
        return parser->tokens->tokens[parser->current++];
    }
    return NULL;
}

/* 匹配 Token 类型 */
bool chim_parser_match(chim_parser_t* parser, chim_token_type_t type) {
    chim_token_t* current = chim_parser_current(parser);
    if (current && current->type == type) {
        chim_parser_advance(parser);
        return true;
    }
    return false;
}

/* 匹配任意 Token 类型 */
bool chim_parser_match_any(chim_parser_t* parser, const chim_token_type_t* types, size_t count) {
    chim_token_t* current = chim_parser_current(parser);
    if (!current) return false;

    for (size_t i = 0; i < count; i++) {
        if (current->type == types[i]) {
            chim_parser_advance(parser);
            return true;
        }
    }
    return false;
}

/* 期望 Token */
void chim_parser_expect(chim_parser_t* parser, chim_token_type_t type, const char* expected_name) {
    chim_token_t* current = chim_parser_current(parser);

    if (!current || current->type != type) {
        chim_syntax_error_report(parser->diagnostics,
            current ? &current->location : &(chim_location_t){NULL, 0, 0, 0},
            expected_name,
            current ? chim_token_type_name(current->type) : "EOF",
            "Expected");
        parser->error_recovery = true;
    } else {
        chim_parser_advance(parser);
    }
}

/* 检查是否到达末尾 */
bool chim_parser_at_end(chim_parser_t* parser) {
    chim_token_t* current = chim_parser_current(parser);
    return !current || current->type == CHIM_TOKEN_EOF;
}

/* 同步到下一个同步点 */
void chim_parser_synchronize(chim_parser_t* parser) {
    parser->error_recovery = false;

    static const chim_token_type_t SYNC_TOKENS[] = {
        CHIM_TOKEN_EOF,
        CHIM_TOKEN_KW_LET,
        CHIM_TOKEN_KW_VAR,
        CHIM_TOKEN_KW_CONST,
        CHIM_TOKEN_KW_FN,
        CHIM_TOKEN_KW_IF,
        CHIM_TOKEN_KW_FOR,
        CHIM_TOKEN_KW_WHILE,
        CHIM_TOKEN_KW_MATCH,
        CHIM_TOKEN_KW_RETURN,
    };

    while (!chim_parser_at_end(parser)) {
        chim_token_t* current = chim_parser_current(parser);

        /* 遇到同步 Token 时停止 */
        for (size_t i = 0; i < CHIM_ARRAY_SIZE(SYNC_TOKENS); i++) {
            if (current->type == SYNC_TOKENS[i]) {
                return;
            }
        }

        chim_parser_advance(parser);
    }
}

/* 恢复解析 */
bool chim_parser_recover_to(chim_parser_t* parser, const chim_token_type_t* sync_tokens, size_t count) {
    parser->error_recovery = false;

    while (!chim_parser_at_end(parser)) {
        chim_token_t* current = chim_parser_current(parser);

        for (size_t i = 0; i < count; i++) {
            if (current->type == sync_tokens[i]) {
                return true;
            }
        }

        chim_parser_advance(parser);
    }

    return false;
}

/* 解析入口 */
chim_ast_node_t* chim_parser_parse(chim_parser_t* parser) {
    return chim_parser_parse_program(parser);
}

/* 解析程序 */
chim_ast_node_t* chim_parser_parse_program(chim_parser_t* parser) {
    chim_location_t loc = {NULL, 1, 1, 0};
    chim_ast_node_t* program = chim_ast_create_program(&loc);

    while (!chim_parser_at_end(parser)) {
        if (parser->error_recovery) {
            chim_parser_synchronize(parser);
        }

        chim_token_t* current = chim_parser_current(parser);

        if (current->type == CHIM_TOKEN_KW_FN) {
            chim_ast_node_t* function = chim_parser_parse_function(parser);
            if (function) {
                chim_ast_program_add_function(program, function);
            }
        } else if (current->type == CHIM_TOKEN_KW_LET ||
                   current->type == CHIM_TOKEN_KW_VAR ||
                   current->type == CHIM_TOKEN_KW_CONST) {
            chim_ast_node_t* decl = NULL;
            if (current->type == CHIM_TOKEN_KW_LET) {
                decl = chim_parser_parse_variable_decl(parser);
            } else if (current->type == CHIM_TOKEN_KW_CONST) {
                decl = chim_parser_parse_const_decl(parser);
            }
            if (decl) {
                chim_ast_program_add_declaration(program, decl);
            }
        } else if (current->type == CHIM_TOKEN_EOF) {
            break;
        } else {
            /* 未知声明，尝试跳过 */
            chim_diag_warning(parser->diagnostics,
                "Unknown top-level declaration at line %d", current->location.line);
            chim_parser_advance(parser);
        }
    }

    return program;
}

/* 解析函数 */
chim_ast_node_t* chim_parser_parse_function(chim_parser_t* parser) {
    chim_location_t start_loc = {NULL, 0, 0, 0};

    if (parser->current > 0) {
        chim_token_t* prev = parser->tokens->tokens[parser->current - 1];
        start_loc = prev->location;
    }

    /* fn 关键字 */
    chim_parser_expect(parser, CHIM_TOKEN_KW_FN, "fn");

    /* 函数名 */
    chim_token_t* name_token = chim_parser_current(parser);
    if (!name_token || name_token->type != CHIM_TOKEN_IDENTIFIER) {
        chim_parser_expect(parser, CHIM_TOKEN_IDENTIFIER, "function name");
        return NULL;
    }
    chim_parser_advance(parser);

    /* 参数列表 */
    chim_parser_expect(parser, CHIM_TOKEN_LPAREN, "(");

    chim_ast_node_t** params = NULL;
    size_t num_params = 0;

    if (!chim_parser_match(parser, CHIM_TOKEN_RPAREN)) {
        while (true) {
            chim_token_t* param_name = chim_parser_current(parser);
            if (!param_name || param_name->type != CHIM_TOKEN_IDENTIFIER) {
                chim_parser_expect(parser, CHIM_TOKEN_IDENTIFIER, "parameter name");
                break;
            }
            chim_parser_advance(parser);

            chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");

            chim_ast_node_t* param_type = chim_parser_parse_type(parser);

            chim_ast_node_t* param = chim_ast_create_parameter(&param_name->location,
                param_name->value.identifier, param_type);

            params = (chim_ast_node_t**)chim_realloc(params, (num_params + 1) * sizeof(chim_ast_node_t*));
            params[num_params++] = param;

            if (!chim_parser_match(parser, CHIM_TOKEN_COMMA)) {
                break;
            }
        }

        chim_parser_expect(parser, CHIM_TOKEN_RPAREN, ")");
    }

    /* 返回类型 */
    chim_ast_node_t* return_type = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
        return_type = chim_parser_parse_type(parser);
    }

    /* 函数体 */
    chim_ast_node_t* body = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_ASSIGN)) {
        /* 单行函数体 */
        chim_ast_node_t* expr = chim_parser_parse_expression(parser);

        /* 包装成块 */
        chim_ast_node_t** stmts = (chim_ast_node_t**)chim_alloc(sizeof(chim_ast_node_t*));
        stmts[0] = chim_ast_create_expr_stmt(&expr->base.location, expr);
        body = chim_ast_create_block(&expr->base.location, stmts, 1);
    } else if (chim_parser_match(parser, CHIM_TOKEN_LBRACE)) {
        body = chim_parser_parse_block(parser);
    } else {
        /* 期待函数体 */
        chim_parser_expect(parser, CHIM_TOKEN_ASSIGN, "=");
        return NULL;
    }

    return chim_ast_create_function(&start_loc,
        name_token->value.identifier,
        params, num_params,
        return_type, body);
}

/* 解析变量声明 */
chim_ast_node_t* chim_parser_parse_variable_decl(chim_parser_t* parser) {
    chim_location_t start_loc = {NULL, 0, 0, 0};

    if (parser->current > 0) {
        chim_token_t* prev = parser->tokens->tokens[parser->current - 1];
        start_loc = prev->location;
    }

    bool is_mutable = false;
    if (chim_parser_match(parser, CHIM_TOKEN_KW_LET)) {
        is_mutable = false;
    } else if (chim_parser_match(parser, CHIM_TOKEN_KW_VAR)) {
        is_mutable = true;
    } else {
        return NULL;
    }

    /* 变量名 */
    chim_token_t* name_token = chim_parser_current(parser);
    if (!name_token || name_token->type != CHIM_TOKEN_IDENTIFIER) {
        chim_parser_expect(parser, CHIM_TOKEN_IDENTIFIER, "variable name");
        return NULL;
    }
    chim_parser_advance(parser);

    /* 类型注解 */
    chim_ast_node_t* var_type = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
        var_type = chim_parser_parse_type(parser);
    }

    /* 初始化 */
    chim_ast_node_t* initializer = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_ASSIGN)) {
        initializer = chim_parser_parse_expression(parser);
    }

    return chim_ast_create_variable_decl(&start_loc,
        name_token->value.identifier,
        var_type, initializer, is_mutable);
}

/* 解析常量声明 */
chim_ast_node_t* chim_parser_parse_const_decl(chim_parser_t* parser) {
    chim_location_t start_loc = {NULL, 0, 0, 0};

    if (parser->current > 0) {
        chim_token_t* prev = parser->tokens->tokens[parser->current - 1];
        start_loc = prev->location;
    }

    chim_parser_expect(parser, CHIM_TOKEN_KW_CONST, "const");

    /* 常量名 */
    chim_token_t* name_token = chim_parser_current(parser);
    if (!name_token || name_token->type != CHIM_TOKEN_IDENTIFIER) {
        chim_parser_expect(parser, CHIM_TOKEN_IDENTIFIER, "constant name");
        return NULL;
    }
    chim_parser_advance(parser);

    /* 类型注解 */
    chim_ast_node_t* const_type = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
        const_type = chim_parser_parse_type(parser);
    }

    /* 初始化 */
    chim_ast_node_t* value = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_ASSIGN)) {
        value = chim_parser_parse_expression(parser);
    }

    return chim_ast_create_const_decl(&start_loc,
        name_token->value.identifier,
        const_type, value);
}

/* 解析块 */
chim_ast_node_t* chim_ast_create_block(const chim_location_t* location,
    chim_ast_node_t** statements, size_t num_statements);

/* 解析参数列表 (内部使用) */
chim_ast_node_t** chim_parser_parse_parameters_internal(chim_parser_t* parser, size_t* out_count) {
    chim_ast_node_t** params = NULL;
    *out_count = 0;

    if (!chim_parser_match(parser, CHIM_TOKEN_RPAREN)) {
        while (true) {
            if (parser->error_recovery) {
                break;
            }

            chim_token_t* name_token = chim_parser_current(parser);
            if (!name_token || name_token->type != CHIM_TOKEN_IDENTIFIER) {
                chim_parser_expect(parser, CHIM_TOKEN_IDENTIFIER, "parameter name");
                break;
            }
            chim_parser_advance(parser);

            chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");

            chim_ast_node_t* param_type = chim_parser_parse_type(parser);

            chim_ast_node_t* param = chim_ast_create_parameter(&name_token->location,
                name_token->value.identifier, param_type);

            params = (chim_ast_node_t**)chim_realloc(params, (*out_count + 1) * sizeof(chim_ast_node_t*));
            params[(*out_count)++] = param;

            if (!chim_parser_match(parser, CHIM_TOKEN_COMMA)) {
                break;
            }
        }

        chim_parser_expect(parser, CHIM_TOKEN_RPAREN, ")");
    }

    return params;
}

/* 解析类型 */
chim_ast_node_t* chim_parser_parse_type(chim_parser_t* parser) {
    chim_token_t* current = chim_parser_current(parser);
    chim_location_t loc = current ? current->location : (chim_location_t){NULL, 0, 0, 0};

    if (!current) return NULL;

    /* 函数类型 fn(param_type) -> return_type */
    if (current->type == CHIM_TOKEN_KW_FN) {
        chim_parser_advance(parser);

        chim_parser_expect(parser, CHIM_TOKEN_LPAREN, "(");

        chim_ast_node_t* param_type = NULL;
        if (!chim_parser_match(parser, CHIM_TOKEN_RPAREN)) {
            param_type = chim_parser_parse_type(parser);
            chim_parser_expect(parser, CHIM_TOKEN_RPAREN, ")");
        }

        chim_parser_expect(parser, CHIM_TOKEN_FAT_ARROW, "=>");

        chim_ast_node_t* return_type = chim_parser_parse_type(parser);

        return chim_ast_create_func_type(&loc, param_type, return_type);
    }

    /* 列表类型 [type] */
    if (current->type == CHIM_TOKEN_LBRACKET) {
        chim_parser_advance(parser);
        chim_ast_node_t* elem_type = chim_parser_parse_type(parser);
        chim_parser_expect(parser, CHIM_TOKEN_RBRACKET, "]");
        return chim_ast_create_list_type(&loc, elem_type);
    }

    /* 基本类型或类型名 */
    if (current->type == CHIM_TOKEN_IDENTIFIER) {
        chim_parser_advance(parser);
        return chim_ast_create_type_ref(&loc, current->value.identifier);
    }

    /* void 类型 */
    if (current->type == CHIM_TOKEN_KW_VOID) {
        chim_parser_advance(parser);
        return chim_ast_create_type_ref(&loc, "void");
    }

    /* 错误 */
    chim_parser_expect(parser, CHIM_TOKEN_IDENTIFIER, "type");

    return chim_ast_create_type_ref(&loc, "unknown");
}

/* 解析语句 */
chim_ast_node_t* chim_parser_parse_statement(chim_parser_t* parser) {
    if (parser->error_recovery) {
        chim_parser_synchronize(parser);
    }

    chim_token_t* current = chim_parser_current(parser);
    if (!current) return NULL;

    switch (current->type) {
        case CHIM_TOKEN_KW_IF:
            return chim_parser_parse_if_stmt(parser);
        case CHIM_TOKEN_KW_FOR:
            return chim_parser_parse_for_stmt(parser);
        case CHIM_TOKEN_KW_WHILE:
            return chim_parser_parse_while_stmt(parser);
        case CHIM_TOKEN_KW_MATCH:
            return chim_parser_parse_match_expr(parser);
        case CHIM_TOKEN_KW_RETURN:
            return chim_parser_parse_return_stmt(parser);
        case CHIM_TOKEN_KW_BREAK:
            return chim_parser_parse_break_stmt(parser);
        case CHIM_TOKEN_KW_CONTINUE:
            return chim_parser_parse_continue_stmt(parser);
        case CHIM_TOKEN_LBRACE:
            return chim_parser_parse_block(parser);
        case CHIM_TOKEN_KW_LET:
        case CHIM_TOKEN_KW_VAR:
            return chim_parser_parse_variable_decl(parser);
        default:
            /* 表达式语句 */
            return chim_ast_create_expr_stmt(&current->location,
                chim_parser_parse_expression(parser));
    }
}

/* 解析 if 语句 */
chim_ast_node_t* chim_parser_parse_if_stmt(chim_parser_t* parser) {
    chim_token_t* start = chim_parser_current(parser);
    chim_location_t start_loc = start->location;

    chim_parser_expect(parser, CHIM_TOKEN_KW_IF, "if");

    /* 条件 */
    chim_ast_node_t* condition = chim_parser_parse_expression(parser);

    /* then 分支 */
    chim_ast_node_t* then_branch = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
        then_branch = chim_parser_parse_statement(parser);
    } else {
        chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");
    }

    /* elif 分支 */
    chim_ast_node_t** elif_branches = NULL;
    size_t num_elifs = 0;

    while (chim_parser_match(parser, CHIM_TOKEN_KW_ELIF)) {
        chim_ast_node_t* elif_cond = chim_parser_parse_expression(parser);

        chim_ast_node_t* elif_body = NULL;
        if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
            elif_body = chim_parser_parse_statement(parser);
        } else {
            chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");
        }

        /* 临时创建 if 表达式作为 elif 分支 */
        chim_ast_node_t* elif_expr = chim_ast_create_if_expr(&elif_cond->base.location,
            elif_cond, elif_body, NULL);

        elif_branches = (chim_ast_node_t**)chim_realloc(elif_branches,
            (num_elifs + 1) * sizeof(chim_ast_node_t*));
        elif_branches[num_elifs++] = elif_expr;
    }

    /* else 分支 */
    chim_ast_node_t* else_branch = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_KW_ELSE)) {
        if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
            else_branch = chim_parser_parse_statement(parser);
        } else {
            chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");
        }
    }

    return chim_ast_create_if_stmt(&start_loc, condition, then_branch,
        elif_branches, num_elifs, else_branch);
}

/* 解析 for 语句 */
chim_ast_node_t* chim_parser_parse_for_stmt(chim_parser_t* parser) {
    chim_token_t* start = chim_parser_current(parser);
    chim_location_t start_loc = start->location;

    chim_parser_expect(parser, CHIM_TOKEN_KW_FOR, "for");

    /* 迭代器变量名 */
    chim_token_t* iter_name = chim_parser_current(parser);
    if (!iter_name || iter_name->type != CHIM_TOKEN_IDENTIFIER) {
        chim_parser_expect(parser, CHIM_TOKEN_IDENTIFIER, "iterator name");
        return NULL;
    }
    chim_parser_advance(parser);

    chim_parser_expect(parser, CHIM_TOKEN_KW_IN, "in");

    /* 可迭代表达式 */
    chim_ast_node_t* iterable = chim_parser_parse_expression(parser);

    /* 循环体 */
    chim_ast_node_t* body = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
        body = chim_parser_parse_statement(parser);
    } else {
        chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");
    }

    return chim_ast_create_for_stmt(&start_loc, iter_name->value.identifier, iterable, body);
}

/* 解析 while 语句 */
chim_ast_node_t* chim_parser_parse_while_stmt(chim_parser_t* parser) {
    chim_token_t* start = chim_parser_current(parser);
    chim_location_t start_loc = start->location;

    chim_parser_expect(parser, CHIM_TOKEN_KW_WHILE, "while");

    /* 条件 */
    chim_ast_node_t* condition = chim_parser_parse_expression(parser);

    /* 循环体 */
    chim_ast_node_t* body = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
        body = chim_parser_parse_statement(parser);
    } else {
        chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");
    }

    return chim_ast_create_while_stmt(&start_loc, condition, body);
}

/* 解析 match 表达式 */
chim_ast_node_t* chim_parser_parse_match_expr(chim_parser_t* parser) {
    chim_token_t* start = chim_parser_current(parser);
    chim_location_t start_loc = start->location;

    chim_parser_expect(parser, CHIM_TOKEN_KW_MATCH, "match");

    /* 被匹配的表达式 */
    chim_ast_node_t* matched_expr = chim_parser_parse_expression(parser);

    chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");

    /* cases */
    chim_ast_node_t** cases = NULL;
    size_t num_cases = 0;

    while (true) {
        chim_token_t* current = chim_parser_current(parser);

        if (!current || current->type == CHIM_TOKEN_EOF ||
            current->type == CHIM_TOKEN_RBRACE) {
            break;
        }

        chim_ast_node_t* pattern = chim_parser_parse_pattern(parser);

        chim_parser_expect(parser, CHIM_TOKEN_FAT_ARROW, "=>");

        chim_ast_node_t* body = NULL;
        if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
            body = chim_parser_parse_statement(parser);
        } else {
            body = chim_parser_parse_expression(parser);
        }

        chim_ast_node_t* match_case = chim_ast_create_match_case(&start_loc, pattern, body);

        cases = (chim_ast_node_t**)chim_realloc(cases, (num_cases + 1) * sizeof(chim_ast_node_t*));
        cases[num_cases++] = match_case;
    }

    return chim_ast_create_match_expr(&start_loc, matched_expr, cases, num_cases);
}

/* 解析模式 */
chim_ast_node_t* chim_parser_parse_pattern(chim_parser_t* parser) {
    chim_token_t* current = chim_parser_current(parser);
    if (!current) return NULL;

    /* 通配符 _ */
    if (current->type == CHIM_TOKEN_UNDERSCORE) {
        chim_parser_advance(parser);
        return chim_ast_create_identifier(&current->location, "_");
    }

    /* 下划线标识符 _name */
    if (current->type == CHIM_TOKEN_UNDERSCORE) {
        chim_ast_node_t* underscore = chim_ast_create_identifier(&current->location, "_");
        chim_parser_advance(parser);

        chim_token_t* name = chim_parser_current(parser);
        if (name && name->type == CHIM_TOKEN_IDENTIFIER) {
            chim_parser_advance(parser);
            return chim_ast_create_identifier(&current->location, name->value.identifier);
        }

        return underscore;
    }

    /* 标识符或字面量 */
    if (current->type == CHIM_TOKEN_IDENTIFIER) {
        chim_parser_advance(parser);
        return chim_ast_create_identifier(&current->location, current->value.identifier);
    }

    /* 字面量 */
    if (chim_token_is_literal(current)) {
        chim_ast_node_t* literal = chim_parser_parse_primary_expr(parser);
        return literal;
    }

    /* 默认返回通配符 */
    return chim_ast_create_identifier(&current->location, "_");
}

/* 解析 return 语句 */
chim_ast_node_t* chim_parser_parse_return_stmt(chim_parser_t* parser) {
    chim_token_t* start = chim_parser_current(parser);
    chim_location_t start_loc = start->location;

    chim_parser_expect(parser, CHIM_TOKEN_KW_RETURN, "return");

    chim_ast_node_t* value = NULL;
    if (!chim_parser_match_any(parser, (chim_token_type_t[]){
        CHIM_TOKEN_SEMICOLON, CHIM_TOKEN_EOF, CHIM_TOKEN_RBRACE}, 3)) {
        value = chim_parser_parse_expression(parser);
    }

    return chim_ast_create_return_stmt(&start_loc, value);
}

/* 解析 break 语句 */
chim_ast_node_t* chim_parser_parse_break_stmt(chim_parser_t* parser) {
    chim_token_t* start = chim_parser_current(parser);
    chim_location_t start_loc = start->location;

    chim_parser_expect(parser, CHIM_TOKEN_KW_BREAK, "break");

    return chim_ast_create_break_stmt(&start_loc);
}

/* 解析 continue 语句 */
chim_ast_node_t* chim_parser_parse_continue_stmt(chim_parser_t* parser) {
    chim_token_t* start = chim_parser_current(parser);
    chim_location_t start_loc = start->location;

    chim_parser_expect(parser, CHIM_TOKEN_KW_CONTINUE, "continue");

    return chim_ast_create_continue_stmt(&start_loc);
}

/* 表达式解析 - 使用优先级下降 */

/* 赋值表达式 */
chim_ast_node_t* chim_parser_parse_assignment_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_or_expr(parser);

    if (!left) return NULL;

    if (chim_parser_match(parser, CHIM_TOKEN_ASSIGN)) {
        chim_location_t loc = left->base.location;
        chim_ast_node_t* value = chim_parser_parse_assignment_expr(parser);
        return chim_ast_create_assignment(&loc, left, value);
    }

    return left;
}

/* 或表达式 || */
chim_ast_node_t* chim_parser_parse_or_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_and_expr(parser);

    while (left && chim_parser_match(parser, CHIM_TOKEN_OR)) {
        chim_location_t loc = left->base.location;
        chim_token_t* op = parser->tokens->tokens[parser->current - 1];
        chim_ast_node_t* right = chim_parser_parse_and_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op->type, left, right);
        left = new_node;
    }

    return left;
}

/* 与表达式 && */
chim_ast_node_t* chim_parser_parse_and_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_bitwise_or_expr(parser);

    while (left && chim_parser_match(parser, CHIM_TOKEN_AND)) {
        chim_location_t loc = left->base.location;
        chim_token_t* op = parser->tokens->tokens[parser->current - 1];
        chim_ast_node_t* right = chim_parser_parse_bitwise_or_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op->type, left, right);
        left = new_node;
    }

    return left;
}

/* 位或表达式 | */
chim_ast_node_t* chim_parser_parse_bitwise_or_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_bitwise_xor_expr(parser);

    while (left && chim_parser_match(parser, CHIM_TOKEN_BITOR)) {
        chim_location_t loc = left->base.location;
        chim_token_t* op = parser->tokens->tokens[parser->current - 1];
        chim_ast_node_t* right = chim_parser_parse_bitwise_xor_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op->type, left, right);
        left = new_node;
    }

    return left;
}

/* 位异或表达式 ^ */
chim_ast_node_t* chim_parser_parse_bitwise_xor_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_bitwise_and_expr(parser);

    while (left && chim_parser_match(parser, CHIM_TOKEN_BITXOR)) {
        chim_location_t loc = left->base.location;
        chim_token_t* op = parser->tokens->tokens[parser->current - 1];
        chim_ast_node_t* right = chim_parser_parse_bitwise_and_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op->type, left, right);
        left = new_node;
    }

    return left;
}

/* 位与表达式 & */
chim_ast_node_t* chim_parser_parse_bitwise_and_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_equality_expr(parser);

    while (left && chim_parser_match(parser, CHIM_TOKEN_BITAND)) {
        chim_location_t loc = left->base.location;
        chim_token_t* op = parser->tokens->tokens[parser->current - 1];
        chim_ast_node_t* right = chim_parser_parse_equality_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op->type, left, right);
        left = new_node;
    }

    return left;
}

/* 相等表达式 == != */
chim_ast_node_t* chim_parser_parse_equality_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_relational_expr(parser);

    while (left) {
        chim_token_type_t op_type;
        if (chim_parser_match(parser, CHIM_TOKEN_EQEQ)) {
            op_type = CHIM_TOKEN_EQEQ;
        } else if (chim_parser_match(parser, CHIM_TOKEN_NEQ)) {
            op_type = CHIM_TOKEN_NEQ;
        } else {
            break;
        }

        chim_location_t loc = left->base.location;
        chim_ast_node_t* right = chim_parser_parse_relational_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op_type, left, right);
        left = new_node;
    }

    return left;
}

/* 关系表达式 < <= > >= */
chim_ast_node_t* chim_parser_parse_relational_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_shift_expr(parser);

    while (left) {
        chim_token_type_t op_type;
        if (chim_parser_match(parser, CHIM_TOKEN_LT)) {
            op_type = CHIM_TOKEN_LT;
        } else if (chim_parser_match(parser, CHIM_TOKEN_LE)) {
            op_type = CHIM_TOKEN_LE;
        } else if (chim_parser_match(parser, CHIM_TOKEN_GT)) {
            op_type = CHIM_TOKEN_GT;
        } else if (chim_parser_match(parser, CHIM_TOKEN_GE)) {
            op_type = CHIM_TOKEN_GE;
        } else {
            break;
        }

        chim_location_t loc = left->base.location;
        chim_ast_node_t* right = chim_parser_parse_shift_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op_type, left, right);
        left = new_node;
    }

    return left;
}

/* 移位表达式 << >> */
chim_ast_node_t* chim_parser_parse_shift_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_additive_expr(parser);

    while (left) {
        chim_token_type_t op_type;
        if (chim_parser_match(parser, CHIM_TOKEN_LSHIFT)) {
            op_type = CHIM_TOKEN_LSHIFT;
        } else if (chim_parser_match(parser, CHIM_TOKEN_RSHIFT)) {
            op_type = CHIM_TOKEN_RSHIFT;
        } else {
            break;
        }

        chim_location_t loc = left->base.location;
        chim_ast_node_t* right = chim_parser_parse_additive_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op_type, left, right);
        left = new_node;
    }

    return left;
}

/* 加法表达式 + - */
chim_ast_node_t* chim_parser_parse_additive_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_multiplicative_expr(parser);

    while (left) {
        chim_token_type_t op_type;
        if (chim_parser_match(parser, CHIM_TOKEN_PLUS)) {
            op_type = CHIM_TOKEN_PLUS;
        } else if (chim_parser_match(parser, CHIM_TOKEN_MINUS)) {
            op_type = CHIM_TOKEN_MINUS;
        } else {
            break;
        }

        chim_location_t loc = left->base.location;
        chim_ast_node_t* right = chim_parser_parse_multiplicative_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op_type, left, right);
        left = new_node;
    }

    return left;
}

/* 乘法表达式 * / % */
chim_ast_node_t* chim_parser_parse_multiplicative_expr(chim_parser_t* parser) {
    chim_ast_node_t* left = chim_parser_parse_unary_expr(parser);

    while (left) {
        chim_token_type_t op_type;
        if (chim_parser_match(parser, CHIM_TOKEN_STAR)) {
            op_type = CHIM_TOKEN_STAR;
        } else if (chim_parser_match(parser, CHIM_TOKEN_SLASH)) {
            op_type = CHIM_TOKEN_SLASH;
        } else if (chim_parser_match(parser, CHIM_TOKEN_PERCENT)) {
            op_type = CHIM_TOKEN_PERCENT;
        } else {
            break;
        }

        chim_location_t loc = left->base.location;
        chim_ast_node_t* right = chim_parser_parse_unary_expr(parser);
        chim_ast_node_t* new_node = chim_ast_create_binary_op(&loc, op_type, left, right);
        left = new_node;
    }

    return left;
}

/* 一元表达式 ! - */
chim_ast_node_t* chim_parser_parse_unary_expr(chim_parser_t* parser) {
    chim_token_t* current = chim_parser_current(parser);

    if (!current) return NULL;

    if (current->type == CHIM_TOKEN_NOT ||
        current->type == CHIM_TOKEN_MINUS ||
        current->type == CHIM_TOKEN_BITAND) {
        chim_token_type_t op_type = current->type;
        chim_location_t loc = current->location;
        chim_parser_advance(parser);
        chim_ast_node_t* operand = chim_parser_parse_unary_expr(parser);
        return chim_ast_create_unary_op(&loc, op_type, operand);
    }

    return chim_parser_parse_postfix_expr(parser);
}

/* 后缀表达式 . [] () */
chim_ast_node_t* chim_parser_parse_postfix_expr(chim_parser_t* parser) {
    chim_ast_node_t* primary = chim_parser_parse_primary_expr(parser);

    if (!primary) return NULL;

    while (true) {
        chim_token_t* current = chim_parser_current(parser);

        if (!current) break;

        if (current->type == CHIM_TOKEN_LPAREN) {
            /* 函数调用 */
            chim_location_t loc = primary->base.location;
            chim_parser_advance(parser);

            chim_ast_node_t** args = NULL;
            size_t num_args = 0;

            if (!chim_parser_match(parser, CHIM_TOKEN_RPAREN)) {
                while (true) {
                    chim_ast_node_t* arg = chim_parser_parse_expression(parser);
                    if (!arg) break;

                    args = (chim_ast_node_t**)chim_realloc(args, (num_args + 1) * sizeof(chim_ast_node_t*));
                    args[num_args++] = arg;

                    if (!chim_parser_match(parser, CHIM_TOKEN_COMMA)) {
                        break;
                    }
                }

                chim_parser_expect(parser, CHIM_TOKEN_RPAREN, ")");
            }

            chim_ast_node_t* call = chim_ast_create_call(&loc, primary, args, num_args);
            primary = call;
        }
        else if (current->type == CHIM_TOKEN_LBRACKET) {
            /* 下标访问 */
            chim_location_t loc = primary->base.location;
            chim_parser_advance(parser);

            chim_ast_node_t* index = chim_parser_parse_expression(parser);
            chim_parser_expect(parser, CHIM_TOKEN_RBRACKET, "]");

            chim_ast_node_t* subscript = chim_ast_create_index_access(&loc, primary, index);
            primary = subscript;
        }
        else if (current->type == CHIM_TOKEN_DOT) {
            /* 字段访问 */
            chim_location_t loc = primary->base.location;
            chim_parser_advance(parser);

            chim_token_t* field_name = chim_parser_current(parser);
            if (!field_name || field_name->type != CHIM_TOKEN_IDENTIFIER) {
                chim_parser_expect(parser, CHIM_TOKEN_IDENTIFIER, "field name");
                break;
            }
            chim_parser_advance(parser);

            chim_ast_node_t* field_access = chim_ast_create_field_access(&loc,
                primary, field_name->value.identifier);
            primary = field_access;
        }
        else {
            break;
        }
    }

    return primary;
}

/* 基本表达式 */
chim_ast_node_t* chim_parser_parse_primary_expr(chim_parser_t* parser) {
    chim_token_t* current = chim_parser_current(parser);
    if (!current) return NULL;

    chim_location_t loc = current->location;

    switch (current->type) {
        case CHIM_TOKEN_IDENTIFIER:
            chim_parser_advance(parser);
            return chim_ast_create_identifier(&loc, current->value.identifier);

        case CHIM_TOKEN_INT_LITERAL:
            chim_parser_advance(parser);
            return chim_ast_create_int_literal(&loc, current->value.int_literal);

        case CHIM_TOKEN_FLOAT_LITERAL:
            chim_parser_advance(parser);
            return chim_ast_create_float_literal(&loc, current->value.float_literal);

        case CHIM_TOKEN_STRING_LITERAL:
            chim_parser_advance(parser);
            return chim_ast_create_string_literal(&loc, current->value.string_literal);

        case CHIM_TOKEN_CHAR_LITERAL:
            chim_parser_advance(parser);
            return chim_ast_create_char_literal(&loc, current->value.char_literal);

        case CHIM_TOKEN_KW_TRUE:
            chim_parser_advance(parser);
            return chim_ast_create_bool_literal(&loc, true);

        case CHIM_TOKEN_KW_FALSE:
            chim_parser_advance(parser);
            return chim_ast_create_bool_literal(&loc, false);

        case CHIM_TOKEN_KW_NIL:
            chim_parser_advance(parser);
            return chim_ast_create_nil_literal(&loc);

        case CHIM_TOKEN_LPAREN: {
            chim_parser_advance(parser);

            /* 检查是否是元组 */
            chim_ast_node_t** elems = NULL;
            size_t num_elems = 0;

            if (!chim_parser_match(parser, CHIM_TOKEN_RPAREN)) {
                while (true) {
                    chim_ast_node_t* elem = chim_parser_parse_expression(parser);
                    if (!elem) break;

                    elems = (chim_ast_node_t**)chim_realloc(elems,
                        (num_elems + 1) * sizeof(chim_ast_node_t*));
                    elems[num_elems++] = elem;

                    if (!chim_parser_match(parser, CHIM_TOKEN_COMMA)) {
                        break;
                    }
                }

                chim_parser_expect(parser, CHIM_TOKEN_RPAREN, ")");
            }

            /* 元组 */
            return chim_ast_create_tuple_literal(&loc, elems, num_elems);
        }

        case CHIM_TOKEN_LBRACKET: {
            chim_ast_node_t** elems = NULL;
            size_t num_elems = 0;

            chim_parser_advance(parser);

            if (!chim_parser_match(parser, CHIM_TOKEN_RBRACKET)) {
                while (true) {
                    chim_ast_node_t* elem = chim_parser_parse_expression(parser);
                    if (!elem) break;

                    elems = (chim_ast_node_t**)chim_realloc(elems,
                        (num_elems + 1) * sizeof(chim_ast_node_t*));
                    elems[num_elems++] = elem;

                    if (!chim_parser_match(parser, CHIM_TOKEN_COMMA)) {
                        break;
                    }
                }

                chim_parser_expect(parser, CHIM_TOKEN_RBRACKET, "]");
            }

            return chim_ast_create_list_literal(&loc, elems, num_elems);
        }

        case CHIM_TOKEN_KW_IF: {
            return chim_parser_parse_if_expr(parser);
        }

        case CHIM_TOKEN_KW_MATCH: {
            return chim_parser_parse_match_expr(parser);
        }

        case CHIM_TOKEN_KW_FN: {
            return chim_parser_parse_anonymous_function(parser);
        }

        default:
            chim_diag_error(parser->diagnostics,
                "Unexpected token: %s", chim_token_type_name(current->type));
            chim_parser_advance(parser);
            return NULL;
    }
}

/* 解析 if 表达式 (单行形式) */
chim_ast_node_t* chim_parser_parse_if_expr(chim_parser_t* parser) {
    chim_token_t* start = chim_parser_current(parser);
    chim_location_t start_loc = start->location;

    chim_parser_expect(parser, CHIM_TOKEN_KW_IF, "if");

    /* 条件 */
    chim_ast_node_t* condition = chim_parser_parse_expression(parser);

    /* then 分支 (表达式) */
    chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");
    chim_ast_node_t* then_branch = chim_parser_parse_expression(parser);

    /* else 分支 */
    chim_ast_node_t* else_branch = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_KW_ELSE)) {
        chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");
        else_branch = chim_parser_parse_expression(parser);
    }

    return chim_ast_create_if_expr(&start_loc, condition, then_branch, else_branch);
}

/* 解析匿名函数 */
chim_ast_node_t* chim_parser_parse_anonymous_function(chim_parser_t* parser) {
    chim_token_t* start = chim_parser_current(parser);
    chim_location_t start_loc = start->location;

    chim_parser_expect(parser, CHIM_TOKEN_KW_FN, "fn");

    /* 参数列表 */
    chim_parser_expect(parser, CHIM_TOKEN_LPAREN, "(");

    chim_ast_node_t** params = NULL;
    size_t num_params = 0;

    if (!chim_parser_match(parser, CHIM_TOKEN_RPAREN)) {
        while (true) {
            chim_token_t* param_name = chim_parser_current(parser);
            if (!param_name || param_name->type != CHIM_TOKEN_IDENTIFIER) {
                chim_parser_expect(parser, CHIM_TOKEN_IDENTIFIER, "parameter name");
                break;
            }
            chim_parser_advance(parser);

            chim_parser_expect(parser, CHIM_TOKEN_COLON, ":");

            chim_ast_node_t* param_type = chim_parser_parse_type(parser);

            chim_ast_node_t* param = chim_ast_create_parameter(&param_name->location,
                param_name->value.identifier, param_type);

            params = (chim_ast_node_t**)chim_realloc(params, (num_params + 1) * sizeof(chim_ast_node_t*));
            params[num_params++] = param;

            if (!chim_parser_match(parser, CHIM_TOKEN_COMMA)) {
                break;
            }
        }

        chim_parser_expect(parser, CHIM_TOKEN_RPAREN, ")");
    }

    /* 返回类型 */
    chim_ast_node_t* return_type = NULL;
    if (chim_parser_match(parser, CHIM_TOKEN_ASSIGN)) {
        return_type = NULL;
    } else if (chim_parser_match(parser, CHIM_TOKEN_COLON)) {
        return_type = chim_parser_parse_type(parser);
        chim_parser_expect(parser, CHIM_TOKEN_ASSIGN, "=");
    }

    /* 函数体 (表达式形式) */
    chim_ast_node_t* body = chim_parser_parse_expression(parser);

    /* 创建函数节点，但不指定名称 */
    chim_ast_node_t* func = chim_ast_create_function(&start_loc,
        NULL, params, num_params, return_type, body);

    return func;
}
