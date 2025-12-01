#!/usr/bin/env python3
import sys
sys.path.append('../compiler')

from lexer import Lexer
from parser_ import Parser
from semantic import SemanticAnalyzer

with open('test_simple_group.chim', 'r', encoding='utf-8') as f:
    source = f.read()

print("源代码:")
print(source)
print("\n" + "=" * 60)

lexer = Lexer(source)
tokens = lexer.tokenize()

print(f"\n词法分析: {len(tokens)} tokens")

parser = Parser(tokens)
ast = parser.parse()

print(f"语法分析: {len(ast.statements)} statements")

analyzer = SemanticAnalyzer(dialect=parser.dialect)
success = analyzer.analyze(ast)

if success:
    print("\n✓ 语义分析通过!")
    print(f"组信息: {analyzer.lifetime_analyzer.group_vars}")
else:
    print("\n✗ 错误:")
    for err in analyzer.get_errors():
        print(f"  - {err}")
