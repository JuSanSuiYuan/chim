#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Rust前端 vs Python前端性能对比
"""
import sys
import time
sys.path.append('../compiler')

from lexer import Lexer as PythonLexer
from ffi import ffi

# 测试源代码
test_code = """system

fn process_data(size: 整数) -> 整数:
    令 total := 0
    令 snap_value := 0
    
    组 临时缓冲:
        令 buffer := 数组(1, 2, 3, 4, 5)
        令 temp := 数组(10, 20, 30)
        设 total = buffer[0] + temp[0]
        令 snapshot_data := 快照 buffer
        输出("快照创建:", snapshot_data)
        设 snap_value = snapshot_data[0]
    
    输出("临时缓冲已释放，总和:", total)
    输出("快照值:", snap_value)
    返回 total

fn nested_groups():
    组 外层:
        令 outer_val := 100
        输出("外层值:", outer_val)
        
        组 内层:
            令 inner_val := 200
            令 h := 句柄 outer_val
            输出("内层通过句柄访问外层:", h)
            输出("内层值:", inner_val)
        
        输出("内层已释放")
    
    输出("外层已释放")

fn 主():
    输出("===== Chim 组生命周期示例 =====")
    输出("")
    
    输出("1. 基础组块:")
    令 result := process_data(5)
    输出("返回值:", result)
    输出("")
    
    输出("2. 嵌套组块:")
    nested_groups()
    输出("")
    
    输出("===== 完成 =====")
"""

def benchmark_python_lexer(code, iterations=100):
    """测试Python词法分析器"""
    start = time.time()
    for _ in range(iterations):
        lexer = PythonLexer(code)
        tokens = lexer.tokenize()
    end = time.time()
    return end - start, len(tokens)

def benchmark_rust_lexer(code, iterations=100):
    """测试Rust词法分析器"""
    if not ffi.has_frontend():
        return None, 0
    
    src_bytes = code.encode('utf-8')
    start = time.time()
    for _ in range(iterations):
        result = ffi.lex(src_bytes)
    end = time.time()
    return end - start, len(result.split('\n')) if result else 0

def main():
    print("=" * 60)
    print("Chim 编译器前端性能对比测试")
    print("=" * 60)
    print(f"\n测试代码长度: {len(test_code)} 字符")
    print(f"测试代码行数: {len(test_code.splitlines())} 行")
    
    iterations = 100
    print(f"\n每个测试运行 {iterations} 次迭代\n")
    
    # Python前端测试
    print("[1] Python 前端词法分析...")
    py_time, py_tokens = benchmark_python_lexer(test_code, iterations)
    print(f"    ✓ 耗时: {py_time:.4f}秒")
    print(f"    ✓ 平均: {py_time/iterations*1000:.2f}ms/次")
    print(f"    ✓ Tokens: {py_tokens}")
    
    # Rust前端测试
    print("\n[2] Rust 前端词法分析...")
    if ffi.has_frontend():
        rust_time, rust_tokens = benchmark_rust_lexer(test_code, iterations)
        if rust_time is not None:
            print(f"    ✓ 耗时: {rust_time:.4f}秒")
            print(f"    ✓ 平均: {rust_time/iterations*1000:.2f}ms/次")
            print(f"    ✓ Tokens: {rust_tokens}")
            
            # 性能对比
            speedup = py_time / rust_time if rust_time > 0 else 0
            print(f"\n{'='*60}")
            print(f"🚀 性能提升: {speedup:.2f}x")
            print(f"   Rust前端比Python前端快 {speedup:.2f} 倍!")
            print(f"{'='*60}")
        else:
            print("    ✗ Rust前端测试失败")
    else:
        print("    ✗ Rust前端未加载")
        print("    提示: 请先编译 Rust 前端")

if __name__ == '__main__':
    main()
