#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试Zig后端代码生成
"""
import sys
sys.path.append('../compiler')

from lexer import Lexer
from parser_ import Parser
from zig_codegen import generate_zig_code

# 测试代码
test_code = """system

fn add(a: 整数, b: 整数) -> 整数:
    返回 a + b

fn 主():
    令 x := 10
    令 y := 20
    令 result := add(x, y)
    输出("结果:", result)
"""

print("=" * 60)
print("Zig 后端代码生成测试")
print("=" * 60)

# 词法分析
print("\n[1] 词法分析...")
lexer = Lexer(test_code)
tokens = lexer.tokenize()
print(f"    ✓ {len(tokens)} tokens")

# 语法分析
print("\n[2] 语法分析...")
parser = Parser(tokens)
ast = parser.parse()
print(f"    ✓ {len(ast.statements)} 顶级语句")

# Zig代码生成
print("\n[3] Zig代码生成...")
zig_code = generate_zig_code(ast, parser.dialect)
print(f"    ✓ {len(zig_code.splitlines())} 行代码生成")

print("\n" + "=" * 60)
print("生成的Zig代码:")
print("=" * 60)
print(zig_code)
print("=" * 60)

# 保存到文件
output_file = "generated_test.zig"
with open(output_file, 'w', encoding='utf-8') as f:
    f.write(zig_code)

print(f"\n✓ 代码已保存到 {output_file}")
print("\n提示: 使用 'zig build-exe generated_test.zig' 编译")
