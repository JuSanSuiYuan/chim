#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Chim 组生命周期功能测试
"""
import sys
sys.path.append('../compiler')

from lexer import Lexer
from parser_ import Parser
from semantic import SemanticAnalyzer

def test_group_lifetime():
    """测试组生命周期功能"""
    print("=" * 60)
    print("Chim 组生命周期功能测试")
    print("=" * 60)
    
    # 读取测试文件
    with open('test_group_lifetime.chim', 'r', encoding='utf-8') as f:
        source = f.read()
    
    print("\n[1] 词法分析...")
    lexer = Lexer(source)
    tokens = lexer.tokenize()
    print(f"✓ 生成 {len(tokens)} 个token")
    
    print("\n[2] 语法分析...")
    parser = Parser(tokens)
    ast = parser.parse()
    print(f"✓ 方言: {parser.dialect}")
    print(f"✓ 生成 {len(ast.statements)} 个顶级语句")
    
    # 打印AST节点类型
    print("\n  AST节点:")
    for i, stmt in enumerate(ast.statements, 1):
        print(f"    {i}. {stmt.__class__.__name__}: {stmt}")
    
    print("\n[3] 语义分析...")
    analyzer = SemanticAnalyzer(dialect=parser.dialect)
    success = analyzer.analyze(ast)
    
    if success:
        print("✓ 语义分析通过!")
        
        # 打印生命周期信息
        print("\n[4] 生命周期信息:")
        lifetime_info = analyzer.lifetime_analyzer.var_lifetimes
        if lifetime_info:
            for var_name, lifetime in lifetime_info.items():
                print(f"  - {var_name}: {lifetime}")
        else:
            print("  (无生命周期信息)")
        
        # 打印组信息
        print("\n[5] 组块信息:")
        group_info = analyzer.lifetime_analyzer.group_vars
        if group_info:
            for group_name, vars_set in group_info.items():
                print(f"  - 组 '{group_name}': {vars_set}")
        else:
            print("  (无组块信息)")
    else:
        print("✗ 语义分析失败!")
        print("\n错误列表:")
        for error in analyzer.get_errors():
            print(f"  - {error}")
    
    print("\n" + "=" * 60)
    print("测试完成!")
    print("=" * 60)
    
    return success

if __name__ == '__main__':
    success = test_group_lifetime()
    sys.exit(0 if success else 1)
