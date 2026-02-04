/**
 * @file token.h
 * @brief Chim 3.1 词法单元定义
 */

#ifndef CHIM_TOKEN_H
#define CHIM_TOKEN_H

#include "common.h"

/* Token 类型枚举 */
typedef enum {
    /* 特殊 Token */
    CHIM_TOKEN_EOF = 0,
    CHIM_TOKEN_ERROR,

    /* 标识符和字面量 */
    CHIM_TOKEN_IDENTIFIER,
    CHIM_TOKEN_INT_LITERAL,
    CHIM_TOKEN_FLOAT_LITERAL,
    CHIM_TOKEN_STRING_LITERAL,
    CHIM_TOKEN_CHAR_LITERAL,

    /* 关键字 */
    CHIM_TOKEN_KW_LET,        /* let */
    CHIM_TOKEN_KW_VAR,        /* var */
    CHIM_TOKEN_KW_CONST,      /* const */
    CHIM_TOKEN_KW_FN,         /* fn */
    CHIM_TOKEN_KW_IF,         /* if */
    CHIM_TOKEN_KW_ELIF,       /* elif */
    CHIM_TOKEN_KW_ELSE,       /* else */
    CHIM_TOKEN_KW_FOR,        /* for */
    CHIM_TOKEN_KW_IN,        /* in */
    CHIM_TOKEN_KW_WHILE,      /* while */
    CHIM_TOKEN_KW_MATCH,      /* match */
    CHIM_TOKEN_KW_RETURN,     /* return */
    CHIM_TOKEN_KW_BREAK,      /* break */
    CHIM_TOKEN_KW_CONTINUE,   /* continue */
    CHIM_TOKEN_KW_TRUE,       /* true */
    CHIM_TOKEN_KW_FALSE,      /* false */
    CHIM_TOKEN_KW_NIL,        /* nil */
    CHIM_TOKEN_KW_VOID,       /* void */
    CHIM_TOKEN_KW_TYPE,       /* type */

    /* 运算符 */
    CHIM_TOKEN_PLUS,         /* + */
    CHIM_TOKEN_MINUS,        /* - */
    CHIM_TOKEN_STAR,         /* * */
    CHIM_TOKEN_SLASH,        /* / */
    CHIM_TOKEN_PERCENT,      /* % */

    /* 比较运算符 */
    CHIM_TOKEN_EQEQ,         /* == */
    CHIM_TOKEN_NEQ,          /* != */
    CHIM_TOKEN_LT,           /* < */
    CHIM_TOKEN_LE,           /* <= */
    CHIM_TOKEN_GT,           /* > */
    CHIM_TOKEN_GE,           /* >= */

    /* 逻辑运算符 */
    CHIM_TOKEN_AND,          /* && */
    CHIM_TOKEN_OR,           /* || */
    CHIM_TOKEN_NOT,          /* ! */

    /* 位运算符 */
    CHIM_TOKEN_BITAND,       /* & */
    CHIM_TOKEN_BITOR,        /* | */
    CHIM_TOKEN_BITXOR,       /* ^ */
    CHIM_TOKEN_LSHIFT,       /* << */
    CHIM_TOKEN_RSHIFT,       /* >> */

    /* 其他符号 */
    CHIM_TOKEN_ASSIGN,       /* = */
    CHIM_TOKEN_COLON,        /* : */
    CHIM_TOKEN_COMMA,        /* , */
    CHIM_TOKEN_SEMICOLON,    /* ; */
    CHIM_TOKEN_DOT,          /* . */
    CHIM_TOKEN_LPAREN,       /* ( */
    CHIM_TOKEN_RPAREN,       /* ) */
    CHIM_TOKEN_LBRACKET,     /* [ */
    CHIM_TOKEN_RBRACKET,     /* ] */
    CHIM_TOKEN_LBRACE,       /* { */
    CHIM_TOKEN_RBRACE,       /* } */
    CHIM_TOKEN_ARROW,        /* -> (保留) */
    CHIM_TOKEN_FAT_ARROW,    /* => */
    CHIM_TOKEN_RANGE,        /* .. */
    CHIM_TOKEN_UNDERSCORE,   /* _ */

    /* 注释 */
    CHIM_TOKEN_COMMENT,
} chim_token_type_t;

/* Token 结构 */
typedef struct {
    chim_token_type_t type;
    chim_location_t location;
    union {
        const char* identifier;
        int64_t int_literal;
        double float_literal;
        char* string_literal;
        char char_literal;
    } value;
} chim_token_t;

/* Token 数组 */
typedef struct {
    chim_token_t* tokens;
    size_t count;
    size_t capacity;
} chim_token_array_t;

/* Token 关键字映射表 */
typedef struct {
    const char* keyword;
    chim_token_type_t token_type;
} chim_keyword_map_t;

/* Token 名称获取 */
const char* chim_token_type_name(chim_token_type_t type);

/* Token 值获取描述 */
const char* chim_token_value_string(const chim_token_t* token);

/* Token 创建 */
chim_token_t* chim_token_create(chim_token_type_t type, const chim_location_t* location);
chim_token_t* chim_token_create_identifier(const char* identifier, const chim_location_t* location);
chim_token_t* chim_token_create_int(int64_t value, const chim_location_t* location);
chim_token_t* chim_token_create_float(double value, const chim_location_t* location);
chim_token_t* chim_token_create_string(const char* value, const chim_location_t* location);
chim_token_t* chim_token_create_char(char value, const chim_location_t* location);
chim_token_t* chim_token_create_error(const char* message, const chim_location_t* location);

/* Token 销毁 */
void chim_token_destroy(chim_token_t* token);

/* Token 数组操作 */
void chim_token_array_init(chim_token_array_t* array);
void chim_token_array_destroy(chim_token_array_t* array);
void chim_token_array_push(chim_token_array_t* array, const chim_token_t* token);
void chim_token_array_clear(chim_token_array_t* array);

/* Token 克隆 */
chim_token_t* chim_token_clone(const chim_token_t* token);

/* Token 比较 */
bool chim_token_equals(const chim_token_t* a, const chim_token_t* b);

/* Token 关键字检查 */
bool chim_is_keyword(const char* identifier);
chim_token_type_t chim_keyword_lookup(const char* identifier);

/* Token 分类检查 */
bool chim_token_is_literal(const chim_token_t* token);
bool chim_token_is_operator(const chim_token_t* token);
bool chim_token_is_keyword(const chim_token_t* token);
bool chim_token_is_punctuation(const chim_token_t* token);

/* 运算符优先级和结合性 */
typedef struct {
    int precedence;
    bool is_right_associative;
} chim_operator_info_t;

chim_operator_info_t chim_get_operator_info(chim_token_type_t type);

#endif /* CHIM_TOKEN_H */
