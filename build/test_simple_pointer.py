#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""简化测试"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'compiler'))

from lexer import Lexer, TokenType
from parser_ import Parser
from zig_codegen import generate_zig_code

test_code = """
system

fn test() -> 整数:
    设 a := 100
    设 pa := &a
    设 v := pa.*
    返回 v
"""

lexer = Lexer(test_code)
tokens = lexer.tokenize()

print("Tokens:")
for i, tok in enumerate(tokens):
    print(f"{i}: {tok}")

print("\n" + "="*60)
print("开始解析...")

parser = Parser(tokens)
ast = parser.parse()

print(f"解析成功! 生成 {len(ast.statements)} 个语句")

print("\n" + "="*60)
print("生成 Zig 代码...\n")

zig_code = generate_zig_code(ast, dialect='system')
print(zig_code)
