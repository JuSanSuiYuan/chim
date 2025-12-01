#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Zig后端完整流水线测试：编译 Chim -> Zig -> 可执行文件
"""
import sys
import subprocess
import os
sys.path.append('../compiler')

from lexer import Lexer
from parser_ import Parser
from zig_codegen import generate_zig_code

# 测试代码
test_code = """system

fn add(a: 整数, b: 整数) -> 整数:
    返回 a + b

fn 主():
    输出("计算结果:")
    令 x := 10
    令 y := 20
    令 result := add(x, y)
    输出("x =", x)
    输出("y =", y)
    输出("x + y =", result)
"""

def compile_and_run():
    print("=" * 60)
    print("Zig 后端完整流水线测试")
    print("=" * 60)
    
    # 1. Chim -> Zig
    print("\n[1/4] Chim编译到Zig...")
    lexer = Lexer(test_code)
    tokens = lexer.tokenize()
    parser = Parser(tokens)
    ast = parser.parse()
    
    zig_code = generate_zig_code(ast, parser.dialect)
    print(f"    ✓ 生成 {len(zig_code.splitlines())} 行Zig代码")
    
    # 保存Zig代码
    zig_file = "test_add.zig"
    with open(zig_file, 'w', encoding='utf-8') as f:
        f.write(zig_code)
    print(f"    ✓ 保存到 {zig_file}")
    
    # 2. 检查Zig编译器
    print("\n[2/4] 检查Zig编译器...")
    try:
        result = subprocess.run(['zig', 'version'], 
                              capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            print(f"    ✓ Zig {result.stdout.strip()}")
        else:
            print("    ✗ Zig编译器未找到")
            return False
    except Exception as e:
        print(f"    ✗ 无法运行Zig: {e}")
        print("    提示: 请安装Zig编译器 (https://ziglang.org/)")
        return False
    
    # 3. 编译Zig代码
    print("\n[3/4] 编译Zig代码...")
    try:
        result = subprocess.run(['zig', 'build-exe', zig_file, '-O', 'ReleaseFast'],
                              capture_output=True, text=True, timeout=30)
        if result.returncode == 0:
            print("    ✓ 编译成功")
        else:
            print(f"    ✗ 编译失败:\n{result.stderr}")
            return False
    except Exception as e:
        print(f"    ✗ 编译错误: {e}")
        return False
    
    # 4. 运行生成的可执行文件
    print("\n[4/4] 运行可执行文件...")
    exe_name = "test_add.exe" if os.name == 'nt' else "test_add"
    if os.path.exists(exe_name):
        try:
            print("-" * 60)
            result = subprocess.run([f'./{exe_name}' if os.name != 'nt' else exe_name],
                                  capture_output=True, text=True, timeout=5)
            print(result.stdout)
            if result.stderr:
                print(result.stderr)
            print("-" * 60)
            print("    ✓ 执行成功")
            return True
        except Exception as e:
            print(f"    ✗ 执行错误: {e}")
            return False
    else:
        print(f"    ✗ 可执行文件 {exe_name} 未找到")
        return False

if __name__ == '__main__':
    success = compile_and_run()
    
    print("\n" + "=" * 60)
    if success:
        print("✅ 完整流水线测试成功!")
    else:
        print("⚠️  部分测试未通过（可能需要安装Zig编译器）")
    print("=" * 60)
