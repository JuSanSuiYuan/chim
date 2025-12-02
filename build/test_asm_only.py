#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'compiler'))

from lexer import Lexer
from parser_ import Parser
from zig_codegen import generate_zig_code

test_code = """
system

fn 测试汇编():
    asm volatile ("nop")
    返回
"""

print("测试内联汇编...")
lexer = Lexer(test_code)
tokens = lexer.tokenize()

print("\nTokens:")
for i, tok in enumerate(tokens[10:25]):
    print(f"{i+10}: {tok}")

print("\n解析...")
parser = Parser(tokens)
ast = parser.parse()

print(f"\n解析成功! 生成 {len(ast.statements)} 个语句")

zig_code = generate_zig_code(ast, dialect='system')
print("\n生成的Zig代码:\n")
print(zig_code)
