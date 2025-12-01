#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试Zig后端组生命周期代码生成
"""
import sys
sys.path.append('../compiler')

from lexer import Lexer
from parser_ import Parser
from zig_codegen import generate_zig_code

# 测试代码 - 组生命周期
test_code = """system

fn test_group():
    组 临时数据:
        令 x := 100
        令 y := 200
        输出("组内:", x, y)
    
    输出("组外完成")

fn 主():
    test_group()
"""

print("=" * 60)
print("Zig 后端组生命周期测试")
print("=" * 60)

# 编译
lexer = Lexer(test_code)
tokens = lexer.tokenize()
parser = Parser(tokens)
ast = parser.parse()

# 生成Zig代码
zig_code = generate_zig_code(ast, parser.dialect)

print("\n生成的Zig代码:")
print("=" * 60)
print(zig_code)
print("=" * 60)

# 保存
with open('group_lifetime_test.zig', 'w', encoding='utf-8') as f:
    f.write(zig_code)

print("\n✓ 代码已保存到 group_lifetime_test.zig")
