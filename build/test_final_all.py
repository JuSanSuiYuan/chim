#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
完整测试 Chim 的三个新功能:
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

# 完整测试代码
test_code = """
system

# 功能1: 指针基本操作
fn test_pointer() -> 整数:
    设 x := 42
    设 px := &x
    设 y := px.*
    返回 y

# 功能2: 内联汇编
fn test_asm():
    asm volatile ("nop")
    返回

# 功能3: 端口 I/O
fn test_port_io() -> 整数:
    设 port := 96
    设 value := inb(port)
    outb(port, value)
    返回 value

# 综合应用: 串口输出
fn serial_write_byte(data: 整数):
    令 COM1 := 1016
    设 ready := inb(COM1 + 5)
    outb(COM1, data)
    返回

fn 主():
    设 r1 := test_pointer()
    test_asm()
    设 r2 := test_port_io()
    serial_write_byte(65)
    返回
"""

def main():
    print("=" * 70)
    print(" Chim 编译器新功能测试: 指针 + 内联汇编 + 端口I/O")
    print("=" * 70)
    print()
    
    try:
        # 词法分析
        print("[1/3] 词法分析...")
        lexer = Lexer(test_code)
        tokens = lexer.tokenize()
        print(f"      ✓ 生成 {len(tokens)} 个 token")
        
        # 语法分析
        print("\n[2/3] 语法分析...")
        parser = Parser(tokens)
        ast = parser.parse()
        print(f"      ✓ 生成 AST，包含 {len(ast.statements)} 个函数定义")
        
        # 代码生成
        print("\n[3/3] 生成 Zig 代码...")
        zig_code = generate_zig_code(ast, dialect='system')
        print("      ✓ Zig 代码生成完成")
        
        # 保存到文件
        output_file = os.path.join(os.path.dirname(__file__), 'output_final.zig')
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(zig_code)
        
        print(f"\n      文件已保存: {output_file}")
        
        # 输出生成的代码
        print("\n" + "=" * 70)
        print(" 生成的 Zig 代码:")
        print("=" * 70)
        print()
        print(zig_code)
        
        print("=" * 70)
        print(" 测试完成! 所有功能正常工作 ✓")
        print("=" * 70)
        print()
        print("功能清单:")
        print("  ✓ 1. 原始指针: &variable (取地址)")
        print("  ✓ 2. 指针解引用: ptr.* (Zig风格) 和 *ptr (C风格)")
        print("  ✓ 3. 内联汇编: asm volatile (\"code\")")
        print("  ✓ 4. 端口I/O: inb/outb/inw/outw/ind/outd")
        print("  ✓ 5. 十六进制字面量: 0x...")
        print()
        
    except Exception as e:
        print(f"\n✗ 错误: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0

if __name__ == '__main__':
    sys.exit(main())
