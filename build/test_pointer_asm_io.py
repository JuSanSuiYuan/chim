#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试 Chim 的新功能:
1. 原始指针 + 解引用
2. 内联汇编
3. 端口 I/O
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'compiler'))

from lexer import Lexer
from parser_ import Parser
from zig_codegen import generate_zig_code

# 测试代码
test_code = """
system

# 测试 1: 指针基本操作
fn 测试指针() -> 整数:
    设 x := 42
    设 px := &x
    设 y := px.*
    返回 y

# 测试 2: 内联汇编
fn 测试汇编():
    asm volatile ("nop")
    返回

# 测试 3: 端口 I/O
fn 测试端口IO() -> 整数:
    设 port := 96
    设 value := inb(port)
    outb(port, value)
    返回 value

# 测试 4: 综合测试 - 串口输出
fn 串口写字节(data: 整数):
    令 COM1 := 1016
    
    # 等待串口就绪
    设 ready := 0
    ready = inb(COM1 + 5)
    
    # 发送数据
    outb(COM1, data)
    返回

fn 主():
    设 result1 := 测试指针()
    输出("指针测试:", result1)
    
    测试汇编()
    输出("汇编测试完成")
    
    设 result3 := 测试端口IO()
    输出("端口IO测试:", result3)
    
    串口写字节(65)
    输出("串口测试完成")
    返回
"""

def main():
    print("=" * 60)
    print("测试 Chim 新功能: 指针、内联汇编、端口I/O")
    print("=" * 60)
    print()
    
    try:
        # 词法分析
        print("1. 词法分析...")
        lexer = Lexer(test_code)
        tokens = lexer.tokenize()
        print(f"   生成 {len(tokens)} 个 token")
        print()
        
        # 语法分析
        print("2. 语法分析...")
        parser = Parser(tokens)
        ast = parser.parse()
        print(f"   生成 AST，包含 {len(ast.statements)} 个语句")
        print()
        
        # 代码生成
        print("3. 生成 Zig 代码...")
        zig_code = generate_zig_code(ast, dialect='system')
        print()
        
        # 输出生成的代码
        print("=" * 60)
        print("生成的 Zig 代码:")
        print("=" * 60)
        print(zig_code)
        print()
        
        # 保存到文件
        output_file = os.path.join(os.path.dirname(__file__), 'test_pointer_asm_io.zig')
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(zig_code)
        
        print(f"✓ 代码已保存到: {output_file}")
        print()
        print("=" * 60)
        print("测试完成!")
        print("=" * 60)
        
    except Exception as e:
        print(f"✗ 错误: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0

if __name__ == '__main__':
    sys.exit(main())
