#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
启语(Chim)编译器 - 词法分析器
"""
import re
from enum import Enum

class TokenType(Enum):
    # 关键字
    FUNC = 'FUNC'           # 函数/fn
    STRUCT = 'STRUCT'       # 结构体/struct
    ENUM = 'ENUM'           # 枚举/enum
    IMPORT = 'IMPORT'       # 导入/import
    EXPORT = 'EXPORT'       # 导出/export
    PUB = 'PUB'             # 公共/pub
    MODULE = 'MODULE'       # 模块/module
    AS = 'AS'               # 作为/as
    GROUP = 'GROUP'         # 组/group
    DYNAMIC = 'DYNAMIC'     # 动态/dynamic
    DSL = 'DSL'             # DSL/toml
    MUT = 'MUT'             # 可变/mut
    VAR = 'VAR'             # 设/var
    LET = 'LET'             # 令/let
    MATCH = 'MATCH'         # 匹配/match
    CASE = 'CASE'           # 案例/case
    DEFAULT = 'DEFAULT'     # 默认/_
    IF = 'IF'               # 如果/if（兼容模式）
    ELIF = 'ELIF'           # 否则如果/elif（兼容模式）
    ELSE = 'ELSE'           # 否则/else（兼容模式）
    SAFE = 'SAFE'           # 安全/safe
    SYSTEM = 'SYSTEM'       # 系统/system
    COMPAT = 'COMPAT'       # 兼容/compat
    UNSAFE = 'UNSAFE'       # 不安全/unsafe
    FOR = 'FOR'             # 对于/for
    IN = 'IN'               # 在/in
    BREAK = 'BREAK'         # 跳出/break
    CONTINUE = 'CONTINUE'   # 继续/continue
    WHILE = 'WHILE'         # 当/while（兼容模式）
    WHEN = 'WHEN'           # 当/when (guard条件)
    CHAN = 'CHAN'           # 通道/chan
    RETURN = 'RETURN'       # 返回/return
    EXTERN = 'EXTERN'       # 外部/extern
    SNAPSHOT = 'SNAPSHOT'   # 快照/snapshot
    HANDLE = 'HANDLE'       # 句柄/handle
    ARENA = 'ARENA'         # arena (system::arena)
    ASM = 'ASM'             # 汇编/asm (内联汇编)
    VOLATILE = 'VOLATILE'   # volatile (阻止优化)
    
    # 指针相关
    STAR = 'STAR'           # * (指针解引用或类型声明)
    AMPERSAND = 'AMPERSAND' # & (取地址)
    AT = 'AT'               # @ (类型转换或特殊操作)
    
    # 常量
    TRUE = 'TRUE'           # 真/true
    FALSE = 'FALSE'         # 假/false
    NIL = 'NIL'             # 空/nil
    
    # 标识符
    IDENTIFIER = 'IDENTIFIER'
    
    # 字面量
    INTEGER = 'INTEGER'
    FLOAT = 'FLOAT'
    STRING = 'STRING'
    CHAR = 'CHAR'
    
    # 运算符
    PLUS = 'PLUS'
    MINUS = 'MINUS'
    MULTIPLY = 'MULTIPLY'
    DIVIDE = 'DIVIDE'
    MODULO = 'MODULO'
    POWER = 'POWER'
    
    # 比较运算符
    EQ = 'EQ'
    NEQ = 'NEQ'
    GEQ = 'GEQ'
    LEQ = 'LEQ'
    GT = 'GT'
    LT = 'LT'
    
    # 逻辑运算符
    NOT = 'NOT'
    AND = 'AND'
    OR = 'OR'
    
    # 赋值运算符
    ASSIGN = 'ASSIGN'
    COLON_ASSIGN = 'COLON_ASSIGN'  # :=
    
    # 符号
    COLON = 'COLON'
    COMMA = 'COMMA'
    DOT = 'DOT'
    RANGE_INCLUSIVE = 'RANGE_INCLUSIVE'  # ...
    RANGE_EXCLUSIVE = 'RANGE_EXCLUSIVE'  # ..
    SCOPE = 'SCOPE'        # ::
    ARROW = 'ARROW'        # ->
    LEFT_PAREN = 'LEFT_PAREN'
    RIGHT_PAREN = 'RIGHT_PAREN'
    LEFT_BRACKET = 'LEFT_BRACKET'
    RIGHT_BRACKET = 'RIGHT_BRACKET'
    LEFT_BRACE = 'LEFT_BRACE'
    RIGHT_BRACE = 'RIGHT_BRACE'
    CHAN_SEND = 'CHAN_SEND'  # <-
    
    # 其他
    COMMENT = 'COMMENT'
    NEWLINE = 'NEWLINE'
    INDENT = 'INDENT'
    DEDENT = 'DEDENT'
    EOF = 'EOF'

class Token:
    def __init__(self, type_, value, line, column):
        self.type = type_
        self.value = value
        self.line = line
        self.column = column
    
    def __repr__(self):
        return f"Token({self.type}, '{self.value}', {self.line}:{self.column})"

class Lexer:
    def __init__(self, source):
        # 移除UTF-8 BOM（如果存在）
        if source.startswith('\ufeff'):
            source = source[1:]
        
        self.source = source
        self.position = 0
        self.line = 1
        self.column = 1
        self.tokens = []
        self.indent_stack = [0]  # 缩进栈，初始为0
        
        # 关键字映射（中文到英文别名）
        self.keywords = {
            '函数': TokenType.FUNC,
            'fn': TokenType.FUNC,
            '结构体': TokenType.STRUCT,
            'struct': TokenType.STRUCT,
            '枚举': TokenType.ENUM,
            'enum': TokenType.ENUM,
            '导入': TokenType.IMPORT,
            'import': TokenType.IMPORT,
            '导出': TokenType.EXPORT,
            'export': TokenType.EXPORT,
            '公共': TokenType.PUB,
            'pub': TokenType.PUB,
            '模块': TokenType.MODULE,
            'module': TokenType.MODULE,
            '作为': TokenType.AS,
            'as': TokenType.AS,
            '组': TokenType.GROUP,
            'group': TokenType.GROUP,
            '动态': TokenType.DYNAMIC,
            'dynamic': TokenType.DYNAMIC,
            'DSL': TokenType.DSL,
            'toml': TokenType.DSL,
            '可变': TokenType.MUT,
            'mut': TokenType.MUT,
            '设': TokenType.VAR,
            'var': TokenType.VAR,
            '令': TokenType.LET,
            'let': TokenType.LET,
            '匹配': TokenType.MATCH,
            'match': TokenType.MATCH,
            '案例': TokenType.CASE,
            'case': TokenType.CASE,
            '默认': TokenType.DEFAULT,
            '_': TokenType.DEFAULT,  # 下划线默认分支
            '如果': TokenType.IF,
            'if': TokenType.IF,
            '否则如果': TokenType.ELIF,
            'elif': TokenType.ELIF,
            '否则': TokenType.ELSE,
            'else': TokenType.ELSE,
            '安全': TokenType.SAFE,
            'safe': TokenType.SAFE,
            '系统': TokenType.SYSTEM,
            'system': TokenType.SYSTEM,
            '兼容': TokenType.COMPAT,
            'compat': TokenType.COMPAT,
            '不安全': TokenType.UNSAFE,
            'unsafe': TokenType.UNSAFE,
            '对于': TokenType.FOR,
            'for': TokenType.FOR,
            '在': TokenType.IN,
            'in': TokenType.IN,
            '跳出': TokenType.BREAK,
            'break': TokenType.BREAK,
            '继续': TokenType.CONTINUE,
            'continue': TokenType.CONTINUE,
            '当': TokenType.WHILE,
            'while': TokenType.WHILE,
            '当': TokenType.WHEN,  # guard条件
            'when': TokenType.WHEN,
            '通道': TokenType.CHAN,
            'chan': TokenType.CHAN,
            '返回': TokenType.RETURN,
            'return': TokenType.RETURN,
            '外部': TokenType.EXTERN,
            'extern': TokenType.EXTERN,
            '汇编': TokenType.ASM,
            'asm': TokenType.ASM,
            'volatile': TokenType.VOLATILE,
            # 端口I/O内置函数
            'inb': TokenType.IDENTIFIER,  # 保持为标识符，在语义分析中处理
            'outb': TokenType.IDENTIFIER,
            'inw': TokenType.IDENTIFIER,
            'outw': TokenType.IDENTIFIER,
            'ind': TokenType.IDENTIFIER,
            'outd': TokenType.IDENTIFIER,
            '快照': TokenType.SNAPSHOT,
            'snapshot': TokenType.SNAPSHOT,
            '句柄': TokenType.HANDLE,
            'handle': TokenType.HANDLE,
            'arena': TokenType.ARENA,
            '真': TokenType.TRUE,
            'true': TokenType.TRUE,
            '假': TokenType.FALSE,
            'false': TokenType.FALSE,
            '空': TokenType.NIL,
            'nil': TokenType.NIL,
        }
    
    def advance(self):
        self.position += 1
        self.column += 1
        if self.position <= len(self.source) and self.source[self.position - 1] == '\n':
            self.line += 1
            self.column = 1
    
    def peek(self, offset=0):
        pos = self.position + offset
        if pos < len(self.source):
            return self.source[pos]
        return None
    
    def skip_whitespace(self):
        while self.position < len(self.source):
            char = self.peek()
            if char.isspace():
                # 处理换行和缩进
                if char == '\n':
                    self.tokens.append(Token(TokenType.NEWLINE, '\n', self.line, self.column))
                    self.advance()
                    # 计算缩进
                    self._process_indent()
                else:
                    self.advance()
            else:
                break
    
    def _process_indent(self):
        """处理缩进和dedent"""
        indent_level = 0
        while self.position < len(self.source) and self.peek() == ' ':
            indent_level += 1
            self.advance()
        
        # 跳过非空白非注释的行
        if self.position < len(self.source) and self.peek() not in ('#', '//'):
            current_indent = self.indent_stack[-1]
            if indent_level > current_indent:
                self.tokens.append(Token(TokenType.INDENT, ' ' * indent_level, self.line, self.column))
                self.indent_stack.append(indent_level)
            elif indent_level < current_indent:
                # 生成DEDENT
                while self.indent_stack and self.indent_stack[-1] > indent_level:
                    self.tokens.append(Token(TokenType.DEDENT, '', self.line, self.column))
                    self.indent_stack.pop()
                if self.indent_stack and self.indent_stack[-1] != indent_level:
                    raise SyntaxError(f"缩进不匹配: 期望 {self.indent_stack[-1]}, 得到 {indent_level} 行 {self.line}")
    
    def skip_comment(self):
        if self.peek() == '#' or (self.peek() == '/' and self.peek(1) == '/'):
            while self.position < len(self.source) and self.peek() != '\n':
                self.advance()
    
    def tokenize_identifier_or_keyword(self):
        start = self.position
        while self.position < len(self.source):
            char = self.peek()
            # 支持中文、英文、数字、下划线
            if char.isalnum() or char == '_' or (ord(char) >= 0x4e00 and ord(char) <= 0x9fff):
                self.advance()
            else:
                break
        
        value = self.source[start:self.position]
        if value in self.keywords:
            return Token(self.keywords[value], value, self.line, self.column - len(value))
        return Token(TokenType.IDENTIFIER, value, self.line, self.column - len(value))
    
    def tokenize_string(self):
        self.advance()  # 跳过引号
        start = self.position
        while self.position < len(self.source) and self.peek() != '"':
            if self.peek() == '\\':
                self.advance()  # 跳过转义字符
            self.advance()
        
        if self.position >= len(self.source):
            raise SyntaxError(f"未闭合的字符串 行 {self.line}")
        
        value = self.source[start:self.position]
        self.advance()  # 跳过结束引号
        return Token(TokenType.STRING, value, self.line, self.column - len(value) - 2)
    
    def tokenize_char(self):
        self.advance()  # 跳过单引号
        if self.position >= len(self.source):
            raise SyntaxError(f"未闭合的字符字面量 行 {self.line}")
        
        if self.peek() == '\\':
            self.advance()  # 跳过转义字符
        
        if self.position >= len(self.source):
            raise SyntaxError(f"未闭合的字符字面量 行 {self.line}")
        
        value = self.source[self.position]
        self.advance()  # 跳过字符
        
        if self.position >= len(self.source) or self.peek() != "'":
            raise SyntaxError(f"未闭合的字符字面量 行 {self.line}")
        
        self.advance()  # 跳过结束单引号
        return Token(TokenType.CHAR, value, self.line, self.column - len(value) - 2)
    
    def tokenize_number(self):
        start = self.position
        has_dot = False
        
        # 检查十六进制数 0x...
        if self.peek() == '0' and self.peek(1) and self.peek(1) in ('x', 'X'):
            self.advance()  # consume '0'
            self.advance()  # consume 'x'
            hex_start = self.position
            while self.position < len(self.source):
                char = self.peek()
                if char and char in '0123456789abcdefABCDEF':
                    self.advance()
                else:
                    break
            hex_str = self.source[hex_start:self.position]
            if not hex_str:
                raise SyntaxError(f"无效的十六进制数 行 {self.line}")
            value = int(hex_str, 16)
            return Token(TokenType.INTEGER, value, self.line, self.column - (self.position - start))
        
        # 普通十进制数
        while self.position < len(self.source):
            char = self.peek()
            if char == '.':
                if has_dot:
                    break  # 已经有小数点了
                has_dot = True
            elif not char.isdigit():
                break
            self.advance()
        
        value = self.source[start:self.position]
        if has_dot:
            return Token(TokenType.FLOAT, float(value), self.line, self.column - len(value))
        return Token(TokenType.INTEGER, int(value), self.line, self.column - len(value))
    
    def tokenize(self):
        """将源代码转换为标记流"""
        while self.position < len(self.source):
            self.skip_whitespace()
            self.skip_comment()
            
            if self.position >= len(self.source):
                break
            
            char = self.peek()
            
            # 标识符或关键字（支持中文）
            if char.isalpha() or char == '_' or (ord(char) >= 0x4e00 and ord(char) <= 0x9fff):
                token = self.tokenize_identifier_or_keyword()
                self.tokens.append(token)
            # 数字
            elif char.isdigit():
                token = self.tokenize_number()
                self.tokens.append(token)
            # 字符串
            elif char == '"':
                token = self.tokenize_string()
                self.tokens.append(token)
            # 字符
            elif char == "'":
                token = self.tokenize_char()
                self.tokens.append(token)
            # 运算符和符号
            elif char == '+':
                self.advance()
                self.tokens.append(Token(TokenType.PLUS, '+', self.line, self.column - 1))
            elif char == '-':
                self.advance()
                if self.peek() == '>':
                    self.advance()
                    self.tokens.append(Token(TokenType.ARROW, '->', self.line, self.column - 2))
                else:
                    self.tokens.append(Token(TokenType.MINUS, '-', self.line, self.column - 1))
            elif char == '*':
                self.advance()
                if self.peek() == '*':
                    self.advance()
                    self.tokens.append(Token(TokenType.POWER, '**', self.line, self.column - 2))
                else:
                    # * 可以是乘法或指针，通过上下文区分
                    self.tokens.append(Token(TokenType.STAR, '*', self.line, self.column - 1))
            elif char == '/':
                self.advance()
                if self.peek() == '/':
                    self.skip_comment()
                else:
                    self.tokens.append(Token(TokenType.DIVIDE, '/', self.line, self.column - 1))
            elif char == '%':
                self.advance()
                self.tokens.append(Token(TokenType.MODULO, '%', self.line, self.column - 1))
            elif char == '=':
                self.advance()
                if self.peek() == '=':
                    self.advance()
                    self.tokens.append(Token(TokenType.EQ, '==', self.line, self.column - 2))
                else:
                    self.tokens.append(Token(TokenType.ASSIGN, '=', self.line, self.column - 1))
            elif char == '!':
                self.advance()
                if self.peek() == '=':
                    self.advance()
                    self.tokens.append(Token(TokenType.NEQ, '!=', self.line, self.column - 2))
                else:
                    self.tokens.append(Token(TokenType.NOT, '!', self.line, self.column - 1))
            elif char == '>':
                self.advance()
                if self.peek() == '=':
                    self.advance()
                    self.tokens.append(Token(TokenType.GEQ, '>=', self.line, self.column - 2))
                else:
                    self.tokens.append(Token(TokenType.GT, '>', self.line, self.column - 1))
            elif char == '<':
                self.advance()
                if self.peek() == '-':
                    self.advance()
                    self.tokens.append(Token(TokenType.CHAN_SEND, '<-', self.line, self.column - 2))
                elif self.peek() == '=':
                    self.advance()
                    self.tokens.append(Token(TokenType.LEQ, '<=', self.line, self.column - 2))
                else:
                    self.tokens.append(Token(TokenType.LT, '<', self.line, self.column - 1))
            elif char == '@':
                self.advance()
                self.tokens.append(Token(TokenType.AT, '@', self.line, self.column - 1))
            elif char == '&':
                self.advance()
                if self.peek() == '&':
                    self.advance()
                    self.tokens.append(Token(TokenType.AND, '&&', self.line, self.column - 2))
                else:
                    # 单个&用于取地址
                    self.tokens.append(Token(TokenType.AMPERSAND, '&', self.line, self.column - 1))
            elif char == '|':
                self.advance()
                if self.peek() == '|':
                    self.advance()
                    self.tokens.append(Token(TokenType.OR, '||', self.line, self.column - 2))
                else:
                    # 这可能是match语句的分隔符
                    self.tokens.append(Token(TokenType.OR, '|', self.line, self.column - 1))
            elif char == ':':
                self.advance()
                if self.peek() == '=':
                    self.advance()
                    self.tokens.append(Token(TokenType.COLON_ASSIGN, ':=', self.line, self.column - 2))
                elif self.peek() == ':':
                    self.advance()
                    self.tokens.append(Token(TokenType.SCOPE, '::', self.line, self.column - 2))
                else:
                    self.tokens.append(Token(TokenType.COLON, ':', self.line, self.column - 1))
            elif char == ',':
                self.advance()
                self.tokens.append(Token(TokenType.COMMA, ',', self.line, self.column - 1))
            elif char == '.':
                self.advance()
                if self.peek() == '.' and self.peek(1) == '.':
                    self.advance()
                    self.advance()
                    self.tokens.append(Token(TokenType.RANGE_INCLUSIVE, '...', self.line, self.column - 3))
                elif self.peek() == '.':
                    self.advance()
                    self.tokens.append(Token(TokenType.RANGE_EXCLUSIVE, '..', self.line, self.column - 2))
                else:
                    self.tokens.append(Token(TokenType.DOT, '.', self.line, self.column - 1))
            elif char == '(':
                self.advance()
                self.tokens.append(Token(TokenType.LEFT_PAREN, '(', self.line, self.column - 1))
            elif char == ')':
                self.advance()
                self.tokens.append(Token(TokenType.RIGHT_PAREN, ')', self.line, self.column - 1))
            elif char == '[':
                self.advance()
                self.tokens.append(Token(TokenType.LEFT_BRACKET, '[', self.line, self.column - 1))
            elif char == ']':
                self.advance()
                self.tokens.append(Token(TokenType.RIGHT_BRACKET, ']', self.line, self.column - 1))
            elif char == '{':
                self.advance()
                self.tokens.append(Token(TokenType.LEFT_BRACE, '{', self.line, self.column - 1))
            elif char == '}':
                self.advance()
                self.tokens.append(Token(TokenType.RIGHT_BRACE, '}', self.line, self.column - 1))
            elif char == '\n':
                # 如果还有换行符，跳过它
                self.advance()
            else:
                # 调试输出
                import sys
                sys.stderr.write(f"\n错误字符: repr={repr(char)}, ord={ord(char)}, pos={self.position}\n")
                raise SyntaxError(f"意外的字符 '{char}' 行 {self.line}")
        
        # 处理剩余的dedent
        while len(self.indent_stack) > 1:
            self.tokens.append(Token(TokenType.DEDENT, '', self.line, self.column))
            self.indent_stack.pop()
        
        self.tokens.append(Token(TokenType.EOF, '', self.line, self.column))
        return self.tokens
