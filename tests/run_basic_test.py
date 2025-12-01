#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
sys.path.append('../compiler')

from lexer import Lexer
from parser_ import Parser
from semantic import SemanticAnalyzer

def test_file(filename):
    print(f"\n{'='*60}")
    print(f"测试文件: {filename}")
    print('='*60)
    
    with open(filename, 'r', encoding='utf-8') as f:
        source = f.read()
    
    try:
        # 词法分析
        print("\n[1] 词法分析...")
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        print(f"    ✓ {len(tokens)} tokens")
        
        # 语法分析
        print("\n[2] 语法分析...")
        parser = Parser(tokens)
        ast = parser.parse()
        print(f"    ✓ 方言: {parser.dialect}")
        print(f"    ✓ {len(ast.statements)} 顶级语句")
        
        # 语义分析
        print("\n[3] 语义分析...")
        analyzer = SemanticAnalyzer(dialect=parser.dialect)
        success = analyzer.analyze(ast)
        
        if success:
            print("    ✓ 语义检查通过")
            
            # 生命周期信息
            if analyzer.lifetime_analyzer.var_lifetimes:
                print("\n[4] 生命周期:")
                for var, lt in analyzer.lifetime_analyzer.var_lifetimes.items():
                    print(f"    - {var}: {lt}")
            
            # 组信息
            if analyzer.lifetime_analyzer.group_vars:
                print("\n[5] 组块:")
                for grp, vars in analyzer.lifetime_analyzer.group_vars.items():
                    print(f"    - '{grp}': {vars}")
            
            print(f"\n{'='*60}")
            print("✅ 测试通过!")
            print('='*60)
            return True
        else:
            print("    ✗ 语义错误:")
            for err in analyzer.get_errors():
                print(f"      - {err}")
            return False
            
    except Exception as e:
        print(f"\n❌ 错误: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == '__main__':
    success = test_file('test_basic_group.chim')
    sys.exit(0 if success else 1)
