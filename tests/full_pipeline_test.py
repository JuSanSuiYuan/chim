#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Chim 完整编译流水线测试
词法 -> 语法 -> 语义 -> 代码生成 -> 执行
"""
import sys
import os
sys.path.append('../compiler')

from lexer import Lexer
from parser_ import Parser
from semantic import SemanticAnalyzer
from codegen import generate_code

def compile_and_run(source_code, test_name="test"):
    print(f"\n{'='*60}")
    print(f"测试: {test_name}")
    print('='*60)
    
    print("\n[1] 词法分析...")
    lexer = Lexer(source_code)
    tokens = lexer.tokenize()
    print(f"    ✓ {len(tokens)} tokens")
    
    print("\n[2] 语法分析...")
    parser = Parser(tokens)
    ast = parser.parse()
    print(f"    ✓ {len(ast.statements)} 语句")
    
    print("\n[3] 语义分析...")
    analyzer = SemanticAnalyzer(dialect=parser.dialect)
    if not analyzer.analyze(ast):
        print("    ✗ 语义错误:")
        for err in analyzer.get_errors():
            print(f"      - {err}")
        return False
    print("    ✓ 语义检查通过")
    
    # 显示生命周期信息
    if analyzer.lifetime_analyzer.group_vars:
        print(f"\n    组块信息: {list(analyzer.lifetime_analyzer.group_vars.keys())}")
    
    print("\n[4] 代码生成...")
    code = generate_code(ast, parser.dialect)
    print("    ✓ Python代码生成完成")
    
    # 保存代码
    output_file = f'{test_name}_output.py'
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(code)
    print(f"    ✓ 保存到 {output_file}")
    
    print(f"\n[5] 执行生成的代码...")
    print("-" * 60)
    try:
        # 执行生成的代码
        exec(compile(code, output_file, 'exec'))
        print("-" * 60)
        print("    ✓ 执行成功")
        return True
    except Exception as e:
        print("-" * 60)
        print(f"    ✗ 执行错误: {e}")
        import traceback
        traceback.print_exc()
        return False

# 测试1: 基础组块
test1 = """system

fn test_basic():
    组 data:
        令 x := 42
        输出("组内值:", x)
    输出("组外完成")

fn 主():
    test_basic()
"""

# 测试2: 快照机制
test2 = """system

fn test_snapshot() -> 整数:
    组 temp:
        令 nums := 数组(10, 20, 30)
        令 snap := 快照 nums
        输出("快照:", snap)
    返回 100

fn 主():
    令 result := test_snapshot()
    输出("结果:", result)
"""

# 运行测试
if __name__ == '__main__':
    print("\n" + "🚀 Chim 编译器完整流水线测试" + "\n")
    
    success1 = compile_and_run(test1, "basic_group")
    success2 = compile_and_run(test2, "snapshot")
    
    print("\n" + "="*60)
    if success1 and success2:
        print("✅ 所有测试通过!")
    else:
        print("❌ 部分测试失败")
    print("="*60)
