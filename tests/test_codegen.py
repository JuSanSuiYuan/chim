#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试代码生成
"""
import sys
sys.path.append('../compiler')

from lexer import Lexer
from parser_ import Parser
from codegen import generate_code

source = """system

fn test():
    组 临时:
        令 x := 100
        令 s := 快照 x
        输出(s)
    输出("done")

fn 主():
    test()
"""

print("源代码:")
print(source)
print("\n" + "="*60)

# 词法分析
lexer = Lexer(source)
tokens = lexer.tokenize()

# 语法分析
parser = Parser(tokens)
ast = parser.parse()

# 代码生成
print("\n生成的Python代码:")
print("="*60)
code = generate_code(ast, parser.dialect)
print(code)
print("="*60)

# 保存到文件
with open('generated_test.py', 'w', encoding='utf-8') as f:
    f.write(code)

print("\n✓ 代码已保存到 generated_test.py")
