/**
 * @file lexer.c
 * @brief Chim 3.1 词法分析器实现
 */

#include <ctype.h>
#include <math.h>
#include <stdlib.h>

#include "lexer.h"
#include "token.h"

/* 关键字表 */
static const chim_keyword_map_t KEYWORD_TABLE[] = {
    {"let", CHIM_TOKEN_KW_LET},
    {"var", CHIM_TOKEN_KW_VAR},
    {"const", CHIM_TOKEN_KW_CONST},
    {"fn", CHIM_TOKEN_KW_FN},
    {"if", CHIM_TOKEN_KW_IF},
    {"elif", CHIM_TOKEN_KW_ELIF},
    {"else", CHIM_TOKEN_KW_ELSE},
    {"for", CHIM_TOKEN_KW_FOR},
    {"in", CHIM_TOKEN_KW_IN},
    {"while", CHIM_TOKEN_KW_WHILE},
    {"match", CHIM_TOKEN_KW_MATCH},
    {"return", CHIM_TOKEN_KW_RETURN},
    {"break", CHIM_TOKEN_KW_BREAK},
    {"continue", CHIM_TOKEN_KW_CONTINUE},
    {"true", CHIM_TOKEN_KW_TRUE},
    {"false", CHIM_TOKEN_KW_FALSE},
    {"nil", CHIM_TOKEN_KW_NIL},
    {"void", CHIM_TOKEN_KW_VOID},
    {"type", CHIM_TOKEN_KW_TYPE},
};

/* 运算符表 */
static const struct {
    const char* str;
    chim_token_type_t type;
    int min_len;
} OPERATOR_TABLE[] = {
    {"==", CHIM_TOKEN_EQEQ, 2},
    {"!=", CHIM_TOKEN_NEQ, 2},
    {"<=", CHIM_TOKEN_LE, 2},
    {">=", CHIM_TOKEN_GE, 2},
    {"&&", CHIM_TOKEN_AND, 2},
    {"||", CHIM_TOKEN_OR, 2},
    {"<<", CHIM_TOKEN_LSHIFT, 2},
    {">>", CHIM_TOKEN_RSHIFT, 2},
    {"=>", CHIM_TOKEN_FAT_ARROW, 2},
    {"..", CHIM_TOKEN_RANGE, 2},
    {"+", CHIM_TOKEN_PLUS, 1},
    {"-", CHIM_TOKEN_MINUS, 1},
    {"*", CHIM_TOKEN_STAR, 1},
    {"/", CHIM_TOKEN_SLASH, 1},
    {"%", CHIM_TOKEN_PERCENT, 1},
    {"<", CHIM_TOKEN_LT, 1},
    {">", CHIM_TOKEN_GT, 1},
    {"&", CHIM_TOKEN_BITAND, 1},
    {"|", CHIM_TOKEN_BITOR, 1},
    {"^", CHIM_TOKEN_BITXOR, 1},
    {"!", CHIM_TOKEN_NOT, 1},
    {"=", CHIM_TOKEN_ASSIGN, 1},
    {":", CHIM_TOKEN_COLON, 1},
    {",", CHIM_TOKEN_COMMA, 1},
    {";", CHIM_TOKEN_SEMICOLON, 1},
    {".", CHIM_TOKEN_DOT, 1},
    {"(", CHIM_TOKEN_LPAREN, 1},
    {")", CHIM_TOKEN_RPAREN, 1},
    {"[", CHIM_TOKEN_LBRACKET, 1},
    {"]", CHIM_TOKEN_RBRACKET, 1},
    {"{", CHIM_TOKEN_LBRACE, 1},
    {"}", CHIM_TOKEN_RBRACE, 1},
    {"_", CHIM_TOKEN_UNDERSCORE, 1},
};

/* 词法分析器状态 */
struct chim_lexer_t {
    const char* source;
    size_t source_len;
    size_t position;
    int line;
    int column;
    chim_diagnostics_t* diagnostics;
    chim_token_array_t tokens;
};

/* 创建词法分析器 */
chim_lexer_t* chim_lexer_create(const char* source, chim_diagnostics_t* diag) {
    chim_lexer_t* lexer = (chim_lexer_t*)chim_alloc(sizeof(chim_lexer_t));
    if (!lexer) {
        return NULL;
    }

    lexer->source = source;
    lexer->source_len = strlen(source);
    lexer->position = 0;
    lexer->line = 1;
    lexer->column = 1;
    lexer->diagnostics = diag;

    chim_token_array_init(&lexer->tokens);

    return lexer;
}

/* 销毁词法分析器 */
void chim_lexer_destroy(chim_lexer_t* lexer) {
    if (!lexer) return;

    chim_token_array_destroy(&lexer->tokens);
    chim_free(lexer);
}

/* 获取当前位置 */
static chim_location_t chim_lexer_location(chim_lexer_t* lexer) {
    return chim_location_create(NULL, lexer->line, lexer->column, lexer->position);
}

/* 查看当前字符 */
static char chim_lexer_peek(chim_lexer_t* lexer) {
    if (lexer->position >= lexer->source_len) {
        return '\0';
    }
    return lexer->source[lexer->position];
}

/* 查看下一个字符 */
static char chim_lexer_peek_next(chim_lexer_t* lexer) {
    if (lexer->position + 1 >= lexer->source_len) {
        return '\0';
    }
    return lexer->source[lexer->position + 1];
}

/* 获取当前字符并前进 */
static char chim_lexer_advance(chim_lexer_t* lexer) {
    if (lexer->position >= lexer->source_len) {
        return '\0';
    }

    char c = lexer->source[lexer->position++];

    if (c == '\n') {
        lexer->line++;
        lexer->column = 1;
    } else {
        lexer->column++;
    }

    return c;
}

/* 跳过空白符 */
static void chim_lexer_skip_whitespace(chim_lexer_t* lexer) {
    while (lexer->position < lexer->source_len) {
        char c = chim_lexer_peek(lexer);

        /* 跳过空格和制表符 */
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n') {
            chim_lexer_advance(lexer);
        } else if (c == '\\' && chim_lexer_peek_next(lexer) == '\n') {
            /* 行继续符 */
            chim_lexer_advance(lexer);  /* \ */
            chim_lexer_advance(lexer);  /* \n */
        } else {
            break;
        }
    }
}

/* 跳过注释 */
static void chim_lexer_skip_comment(chim_lexer_t* lexer) {
    char c = chim_lexer_peek(lexer);

    if (c == '#') {
        /* 单行注释 */
        while (lexer->position < lexer->source_len && chim_lexer_peek(lexer) != '\n') {
            chim_lexer_advance(lexer);
        }
    }
}

/* 识别标识符或关键字 */
static chim_token_t* chim_lexer_identifier(chim_lexer_t* lexer) {
    chim_location_t loc = chim_lexer_location(lexer);
    size_t start = lexer->position;

    while (lexer->position < lexer->source_len) {
        char c = chim_lexer_peek(lexer);
        if (!isalnum(c) && c != '_') {
            break;
        }
        chim_lexer_advance(lexer);
    }

    size_t len = lexer->position - start;
    char* identifier = (char*)chim_alloc(len + 1);
    strncpy(identifier, &lexer->source[start], len);
    identifier[len] = '\0';

    /* 查找关键字 */
    chim_token_type_t type = chim_keyword_lookup(identifier);

    if (type == CHIM_TOKEN_IDENTIFIER) {
        return chim_token_create_identifier(identifier, &loc);
    } else {
        chim_free(identifier);
        return chim_token_create(type, &loc);
    }
}

/* 识别数字字面量 */
static chim_token_t* chim_lexer_number(chim_lexer_t* lexer) {
    chim_location_t loc = chim_lexer_location(lexer);
    bool has_dot = false;
    bool has_exp = false;
    size_t start = lexer->position;

    while (lexer->position < lexer->source_len) {
        char c = chim_lexer_peek(lexer);

        if (isdigit(c)) {
            chim_lexer_advance(lexer);
        } else if (c == '.' && !has_dot && !has_exp) {
            has_dot = true;
            chim_lexer_advance(lexer);
        } else if ((c == 'e' || c == 'E') && !has_exp) {
            has_exp = true;
            chim_lexer_advance(lexer);
            if (chim_lexer_peek(lexer) == '+' || chim_lexer_peek(lexer) == '-') {
                chim_lexer_advance(lexer);
            }
        } else {
            break;
        }
    }

    size_t len = lexer->position - start;
    char* num_str = (char*)chim_alloc(len + 1);
    strncpy(num_str, &lexer->source[start], len);
    num_str[len] = '\0';

    chim_token_t* token;

    if (has_dot || has_exp) {
        double value = strtod(num_str, NULL);
        token = chim_token_create_float(value, &loc);
    } else {
        int64_t value = strtoll(num_str, NULL, 10);
        token = chim_token_create_int(value, &loc);
    }

    chim_free(num_str);
    return token;
}

/* 识别字符串字面量 */
static chim_token_t* chim_lexer_string(chim_lexer_t* lexer) {
    chim_location_t loc = chim_lexer_location(lexer);
    char delimiter = chim_lexer_advance(lexer);  /* 跳过引号 */

    char* buffer = (char*)chim_alloc(256);
    size_t buffer_len = 0;
    size_t buffer_capacity = 256;

    while (lexer->position < lexer->source_len) {
        char c = chim_lexer_peek(lexer);

        if (c == delimiter) {
            chim_lexer_advance(lexer);
            break;
        }

        if (c == '\\') {
            chim_lexer_advance(lexer);
            char escape = chim_lexer_advance(lexer);

            switch (escape) {
                case 'n': c = '\n'; break;
                case 't': c = '\t'; break;
                case '\\': c = '\\'; break;
                case '\'': c = '\''; break;
                case '"': c = '"'; break;
                case '$': c = '$'; break;
                default:
                    /* 未知转义序列 */
                    if (isdigit(escape)) {
                        /* 八进制转义 */
                        c = escape - '0';
                        char next = chim_lexer_peek(lexer);
                        if (isdigit(next)) {
                            c = c * 8 + (next - '0');
                            chim_lexer_advance(lexer);
                            next = chim_lexer_peek(lexer);
                            if (isdigit(next)) {
                                c = c * 8 + (next - '0');
                                chim_lexer_advance(lexer);
                            }
                        }
                    } else {
                        c = escape;
                    }
                    break;
            }
        }

        if (buffer_len >= buffer_capacity) {
            buffer_capacity *= 2;
            buffer = (char*)chim_realloc(buffer, buffer_capacity);
        }

        buffer[buffer_len++] = c;
    }

    buffer[buffer_len] = '\0';

    chim_token_t* token = chim_token_create_string(buffer, &loc);

    /* 重新分配为实际大小 */
    char* final_str = (char*)chim_realloc(buffer, buffer_len + 1);
    token->value.string_literal = final_str;

    return token;
}

/* 识别字符字面量 */
static chim_token_t* chim_lexer_char(chim_lexer_t* lexer) {
    chim_location_t loc = chim_lexer_location(lexer);
    chim_lexer_advance(lexer);  /* 跳过单引号 */

    char c = chim_lexer_advance(lexer);

    if (c == '\\') {
        char escape = chim_lexer_advance(lexer);
        switch (escape) {
            case 'n': c = '\n'; break;
            case 't': c = '\t'; break;
            case '\\': c = '\\'; break;
            case '\'': c = '\''; break;
            default: c = escape; break;
        }
    }

    chim_lexer_advance(lexer);  /* 跳过结尾的单引号 */

    return chim_token_create_char(c, &loc);
}

/* 识别运算符 */
static chim_token_t* chim_lexer_operator(chim_lexer_t* lexer) {
    chim_location_t loc = chim_lexer_location(lexer);

    /* 尝试匹配最长可能的运算符 */
    int best_match = -1;
    chim_token_type_t best_type = CHIM_TOKEN_ERROR;

    for (size_t i = 0; i < CHIM_ARRAY_SIZE(OPERATOR_TABLE); i++) {
        const char* op_str = OPERATOR_TABLE[i].str;
        size_t op_len = strlen(op_str);

        if (lexer->position + op_len <= lexer->source_len) {
            if (strncmp(&lexer->source[lexer->position], op_str, op_len) == 0) {
                if ((int)op_len > best_match) {
                    best_match = (int)op_len;
                    best_type = OPERATOR_TABLE[i].type;
                }
            }
        }
    }

    if (best_match > 0) {
        lexer->position += best_match;
        lexer->column += best_match;
        return chim_token_create(best_type, &loc);
    }

    /* 无法识别的字符 */
    char unknown = chim_lexer_advance(lexer);
    char* msg = chim_sprintf("Unknown character: '%c'", unknown);
    chim_token_t* token = chim_token_create_error(msg, &loc);
    chim_free(msg);

    return token;
}

/* 执行词法分析 */
void chim_lexer_tokenize(chim_lexer_t* lexer) {
    while (lexer->position < lexer->source_len) {
        chim_lexer_skip_whitespace(lexer);

        if (lexer->position >= lexer->source_len) {
            break;
        }

        /* 检查注释 */
        if (chim_lexer_peek(lexer) == '#') {
            chim_lexer_skip_comment(lexer);
            continue;
        }

        char c = chim_lexer_peek(lexer);

        /* 标识符或关键字 */
        if (isalpha(c) || c == '_') {
            chim_token_t* token = chim_lexer_identifier(lexer);
            chim_token_array_push(&lexer->tokens, token);
        }
        /* 数字 */
        else if (isdigit(c)) {
            chim_token_t* token = chim_lexer_number(lexer);
            chim_token_array_push(&lexer->tokens, token);
        }
        /* 字符串 */
        else if (c == '"' || c == '\'') {
            if (c == '\'') {
                chim_token_t* token = chim_lexer_char(lexer);
                chim_token_array_push(&lexer->tokens, token);
            } else {
                chim_token_t* token = chim_lexer_string(lexer);
                chim_token_array_push(&lexer->tokens, token);
            }
        }
        /* 运算符 */
        else {
            chim_token_t* token = chim_lexer_operator(lexer);
            chim_token_array_push(&lexer->tokens, token);

            if (token->type == CHIM_TOKEN_ERROR) {
                /* 词法分析错误，继续尝试 */
                continue;
            }
        }
    }

    /* 添加文件结束符 */
    chim_location_t eof_loc = chim_lexer_location(lexer);
    chim_token_t* eof = chim_token_create(CHIM_TOKEN_EOF, &eof_loc);
    chim_token_array_push(&lexer->tokens, eof);
}

/* 获取 Token 数组 */
chim_token_array_t* chim_lexer_get_tokens(chim_lexer_t* lexer) {
    return &lexer->tokens;
}

/* Token 名称获取 */
const char* chim_token_type_name(chim_token_type_t type) {
    switch (type) {
        case CHIM_TOKEN_EOF: return "EOF";
        case CHIM_TOKEN_ERROR: return "ERROR";
        case CHIM_TOKEN_IDENTIFIER: return "IDENTIFIER";
        case CHIM_TOKEN_INT_LITERAL: return "INT_LITERAL";
        case CHIM_TOKEN_FLOAT_LITERAL: return "FLOAT_LITERAL";
        case CHIM_TOKEN_STRING_LITERAL: return "STRING_LITERAL";
        case CHIM_TOKEN_CHAR_LITERAL: return "CHAR_LITERAL";
        case CHIM_TOKEN_KW_LET: return "let";
        case CHIM_TOKEN_KW_VAR: return "var";
        case CHIM_TOKEN_KW_CONST: return "const";
        case CHIM_TOKEN_KW_FN: return "fn";
        case CHIM_TOKEN_KW_IF: return "if";
        case CHIM_TOKEN_KW_ELIF: return "elif";
        case CHIM_TOKEN_KW_ELSE: return "else";
        case CHIM_TOKEN_KW_FOR: return "for";
        case CHIM_TOKEN_KW_IN: return "in";
        case CHIM_TOKEN_KW_WHILE: return "while";
        case CHIM_TOKEN_KW_MATCH: return "match";
        case CHIM_TOKEN_KW_RETURN: return "return";
        case CHIM_TOKEN_KW_BREAK: return "break";
        case CHIM_TOKEN_KW_CONTINUE: return "continue";
        case CHIM_TOKEN_KW_TRUE: return "true";
        case CHIM_TOKEN_KW_FALSE: return "false";
        case CHIM_TOKEN_KW_NIL: return "nil";
        case CHIM_TOKEN_KW_VOID: return "void";
        case CHIM_TOKEN_KW_TYPE: return "type";
        case CHIM_TOKEN_PLUS: return "+";
        case CHIM_TOKEN_MINUS: return "-";
        case CHIM_TOKEN_STAR: return "*";
        case CHIM_TOKEN_SLASH: return "/";
        case CHIM_TOKEN_PERCENT: return "%";
        case CHIM_TOKEN_EQEQ: return "==";
        case CHIM_TOKEN_NEQ: return "!=";
        case CHIM_TOKEN_LT: return "<";
        case CHIM_TOKEN_LE: return "<=";
        case CHIM_TOKEN_GT: return ">";
        case CHIM_TOKEN_GE: return ">=";
        case CHIM_TOKEN_AND: return "&&";
        case CHIM_TOKEN_OR: return "||";
        case CHIM_TOKEN_NOT: return "!";
        case CHIM_TOKEN_BITAND: return "&";
        case CHIM_TOKEN_BITOR: return "|";
        case CHIM_TOKEN_BITXOR: return "^";
        case CHIM_TOKEN_LSHIFT: return "<<";
        case CHIM_TOKEN_RSHIFT: return ">>";
        case CHIM_TOKEN_ASSIGN: return "=";
        case CHIM_TOKEN_COLON: return ":";
        case CHIM_TOKEN_COMMA: return ",";
        case CHIM_TOKEN_SEMICOLON: return ";";
        case CHIM_TOKEN_DOT: return ".";
        case CHIM_TOKEN_LPAREN: return "(";
        case CHIM_TOKEN_RPAREN: return ")";
        case CHIM_TOKEN_LBRACKET: return "[";
        case CHIM_TOKEN_RBRACKET: return "]";
        case CHIM_TOKEN_LBRACE: return "{";
        case CHIM_TOKEN_RBRACE: return "}";
        case CHIM_TOKEN_FAT_ARROW: return "=>";
        case CHIM_TOKEN_RANGE: return "..";
        case CHIM_TOKEN_UNDERSCORE: return "_";
        case CHIM_TOKEN_COMMENT: return "COMMENT";
        default: return "UNKNOWN";
    }
}

/* Token 值字符串 */
const char* chim_token_value_string(const chim_token_t* token) {
    switch (token->type) {
        case CHIM_TOKEN_IDENTIFIER:
            return token->value.identifier;
        case CHIM_TOKEN_INT_LITERAL:
            return "INT";
        case CHIM_TOKEN_FLOAT_LITERAL:
            return "FLOAT";
        case CHIM_TOKEN_STRING_LITERAL:
            return token->value.string_literal;
        case CHIM_TOKEN_CHAR_LITERAL:
            return "CHAR";
        default:
            return chim_token_type_name(token->type);
    }
}

/* Token 创建 */
chim_token_t* chim_token_create(chim_token_type_t type, const chim_location_t* location) {
    chim_token_t* token = (chim_token_t*)chim_alloc(sizeof(chim_token_t));
    if (!token) return NULL;

    token->type = type;
    chim_location_copy(&token->location, location);
    return token;
}

chim_token_t* chim_token_create_identifier(const char* identifier, const chim_location_t* location) {
    chim_token_t* token = chim_token_create(CHIM_TOKEN_IDENTIFIER, location);
    if (token) {
        token->value.identifier = chim_strdup(identifier);
    }
    return token;
}

chim_token_t* chim_token_create_int(int64_t value, const chim_location_t* location) {
    chim_token_t* token = chim_token_create(CHIM_TOKEN_INT_LITERAL, location);
    if (token) {
        token->value.int_literal = value;
    }
    return token;
}

chim_token_t* chim_token_create_float(double value, const chim_location_t* location) {
    chim_token_t* token = chim_token_create(CHIM_TOKEN_FLOAT_LITERAL, location);
    if (token) {
        token->value.float_literal = value;
    }
    return token;
}

chim_token_t* chim_token_create_string(const char* value, const chim_location_t* location) {
    chim_token_t* token = chim_token_create(CHIM_TOKEN_STRING_LITERAL, location);
    if (token) {
        token->value.string_literal = chim_strdup(value);
    }
    return token;
}

chim_token_t* chim_token_create_char(char value, const chim_location_t* location) {
    chim_token_t* token = chim_token_create(CHIM_TOKEN_CHAR_LITERAL, location);
    if (token) {
        token->value.char_literal = value;
    }
    return token;
}

chim_token_t* chim_token_create_error(const char* message, const chim_location_t* location) {
    chim_token_t* token = chim_token_create(CHIM_TOKEN_ERROR, location);
    if (token) {
        token->value.string_literal = chim_strdup(message);
    }
    return token;
}

/* Token 销毁 */
void chim_token_destroy(chim_token_t* token) {
    if (!token) return;

    switch (token->type) {
        case CHIM_TOKEN_IDENTIFIER:
        case CHIM_TOKEN_STRING_LITERAL:
        case CHIM_TOKEN_ERROR:
            chim_free(token->value.string_literal);
            break;
        default:
            break;
    }

    chim_free(token);
}

/* Token 数组操作 */
void chim_token_array_init(chim_token_array_t* array) {
    array->tokens = NULL;
    array->count = 0;
    array->capacity = 0;
}

void chim_token_array_destroy(chim_token_array_t* array) {
    for (size_t i = 0; i < array->count; i++) {
        chim_token_destroy(array->tokens[i]);
    }
    chim_free(array->tokens);
    array->tokens = NULL;
    array->count = 0;
    array->capacity = 0;
}

void chim_token_array_push(chim_token_array_t* array, const chim_token_t* token) {
    if (array->count >= array->capacity) {
        array->capacity = array->capacity ? array->capacity * 2 : 16;
        array->tokens = (chim_token_t**)chim_realloc(array->tokens,
            array->capacity * sizeof(chim_token_t*));
    }
    array->tokens[array->count++] = chim_token_clone(token);
}

void chim_token_array_clear(chim_token_array_t* array) {
    for (size_t i = 0; i < array->count; i++) {
        chim_token_destroy(array->tokens[i]);
    }
    array->count = 0;
}

/* Token 克隆 */
chim_token_t* chim_token_clone(const chim_token_t* token) {
    chim_token_t* clone = (chim_token_t*)chim_alloc(sizeof(chim_token_t));
    if (!clone) return NULL;

    memcpy(clone, token, sizeof(chim_token_t));

    switch (token->type) {
        case CHIM_TOKEN_IDENTIFIER:
        case CHIM_TOKEN_STRING_LITERAL:
        case CHIM_TOKEN_ERROR:
            clone->value.string_literal = chim_strdup(token->value.string_literal);
            break;
        default:
            break;
    }

    return clone;
}

/* Token 比较 */
bool chim_token_equals(const chim_token_t* a, const chim_token_t* b) {
    if (a->type != b->type) return false;

    switch (a->type) {
        case CHIM_TOKEN_IDENTIFIER:
            return strcmp(a->value.identifier, b->value.identifier) == 0;
        case CHIM_TOKEN_INT_LITERAL:
            return a->value.int_literal == b->value.int_literal;
        case CHIM_TOKEN_FLOAT_LITERAL:
            return a->value.float_literal == b->value.float_literal;
        case CHIM_TOKEN_STRING_LITERAL:
            return strcmp(a->value.string_literal, b->value.string_literal) == 0;
        case CHIM_TOKEN_CHAR_LITERAL:
            return a->value.char_literal == b->value.char_literal;
        default:
            return true;
    }
}

/* 关键字查找 */
bool chim_is_keyword(const char* identifier) {
    return chim_keyword_lookup(identifier) != CHIM_TOKEN_IDENTIFIER;
}

chim_token_type_t chim_keyword_lookup(const char* identifier) {
    for (size_t i = 0; i < CHIM_ARRAY_SIZE(KEYWORD_TABLE); i++) {
        if (strcmp(KEYWORD_TABLE[i].keyword, identifier) == 0) {
            return KEYWORD_TABLE[i].token_type;
        }
    }
    return CHIM_TOKEN_IDENTIFIER;
}

/* Token 分类检查 */
bool chim_token_is_literal(const chim_token_t* token) {
    switch (token->type) {
        case CHIM_TOKEN_INT_LITERAL:
        case CHIM_TOKEN_FLOAT_LITERAL:
        case CHIM_TOKEN_STRING_LITERAL:
        case CHIM_TOKEN_CHAR_LITERAL:
        case CHIM_TOKEN_KW_TRUE:
        case CHIM_TOKEN_KW_FALSE:
        case CHIM_TOKEN_KW_NIL:
            return true;
        default:
            return false;
    }
}

bool chim_token_is_operator(const chim_token_t* token) {
    switch (token->type) {
        case CHIM_TOKEN_PLUS:
        case CHIM_TOKEN_MINUS:
        case CHIM_TOKEN_STAR:
        case CHIM_TOKEN_SLASH:
        case CHIM_TOKEN_PERCENT:
        case CHIM_TOKEN_EQEQ:
        case CHIM_TOKEN_NEQ:
        case CHIM_TOKEN_LT:
        case CHIM_TOKEN_LE:
        case CHIM_TOKEN_GT:
        case CHIM_TOKEN_GE:
        case CHIM_TOKEN_AND:
        case CHIM_TOKEN_OR:
        case CHIM_TOKEN_NOT:
        case CHIM_TOKEN_BITAND:
        case CHIM_TOKEN_BITOR:
        case CHIM_TOKEN_BITXOR:
        case CHIM_TOKEN_LSHIFT:
        case CHIM_TOKEN_RSHIFT:
            return true;
        default:
            return false;
    }
}

bool chim_token_is_keyword(const chim_token_t* token) {
    return token->type >= CHIM_TOKEN_KW_LET && token->type <= CHIM_TOKEN_KW_TYPE;
}

bool chim_token_is_punctuation(const chim_token_t* token) {
    switch (token->type) {
        case CHIM_TOKEN_COLON:
        case CHIM_TOKEN_COMMA:
        case CHIM_TOKEN_SEMICOLON:
        case CHIM_TOKEN_DOT:
        case CHIM_TOKEN_LPAREN:
        case CHIM_TOKEN_RPAREN:
        case CHIM_TOKEN_LBRACKET:
        case CHIM_TOKEN_RBRACKET:
        case CHIM_TOKEN_LBRACE:
        case CHIM_TOKEN_RBRACE:
        case CHIM_TOKEN_FAT_ARROW:
        case CHIM_TOKEN_RANGE:
        case CHIM_TOKEN_UNDERSCORE:
            return true;
        default:
            return false;
    }
}

/* 运算符信息 */
chim_operator_info_t chim_get_operator_info(chim_token_type_t type) {
    chim_operator_info_t info = {0, false};

    switch (type) {
        case CHIM_TOKEN_OR:
            info.precedence = 10;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_AND:
            info.precedence = 20;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_BITOR:
            info.precedence = 30;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_BITXOR:
            info.precedence = 40;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_BITAND:
            info.precedence = 50;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_EQEQ:
        case CHIM_TOKEN_NEQ:
            info.precedence = 60;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_LT:
        case CHIM_TOKEN_LE:
        case CHIM_TOKEN_GT:
        case CHIM_TOKEN_GE:
            info.precedence = 70;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_LSHIFT:
        case CHIM_TOKEN_RSHIFT:
            info.precedence = 80;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_PLUS:
        case CHIM_TOKEN_MINUS:
            info.precedence = 90;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_STAR:
        case CHIM_TOKEN_SLASH:
        case CHIM_TOKEN_PERCENT:
            info.precedence = 100;
            info.is_right_associative = false;
            break;
        case CHIM_TOKEN_NOT:
        case CHIM_TOKEN_MINUS:  /* 一元减号 */
            info.precedence = 110;
            info.is_right_associative = true;
            break;
        default:
            info.precedence = 0;
            break;
    }

    return info;
}
