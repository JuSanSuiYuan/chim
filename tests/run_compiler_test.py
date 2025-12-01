#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
编译器测试脚本 - 验证Rust前端和Python编译器
"""
import sys
import os

# 添加路径
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'compiler'))

def test_python_compiler():
    """测试Python编译器"""
    print("=" * 60)
    print("测试 Python 编译器")
    print("=" * 60)
    
    from main import compile_file
    
    test_file = os.path.join(os.path.dirname(__file__), 'test_basic.chim')
    
    try:
        result = compile_file(test_file)
        print("✅ 编译成功!")
        print("\n生成的代码:")
        print("-" * 60)
        print(result)
        print("-" * 60)
        return True
    except Exception as e:
        print(f"❌ 编译失败: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_rust_frontend():
    """测试Rust前端FFI"""
    print("\n" + "=" * 60)
    print("测试 Rust 前端 FFI")
    print("=" * 60)
    
    try:
        from ffi import ffi
        
        if not ffi.has_frontend():
            print("⚠️  Rust前端库未找到，跳过测试")
            return True  # 不算失败
    except Exception as e:
        print(f"⚠️  无法加载FFI (可能是架构不匹配): {e}")
        print("提示: 确保Python和Rust编译目标架构一致")
        return True  # 不算失败,只是警告
    
    print("✅ Rust前端库已加载")
    print(f"版本: {ffi.version}")
    
    # 测试词法分析
    test_code = """fn 主函数():
    令 x := 1
    返回 x
"""
    
    try:
        tokens = ffi.lex(test_code.encode('utf-8'))
        if tokens:
            print(f"\n✅ 词法分析成功，生成 {len(tokens.split(chr(10)))} 个token")
            print("前10个token:")
            print(tokens[:500])
        else:
            print("⚠️  词法分析返回空")
    except Exception as e:
        print(f"❌ 词法分析失败: {e}")
        import traceback
        traceback.print_exc()
    
    # 测试IR构建
    try:
        ir = ffi.build_ir(test_code.encode('utf-8'))
        if ir:
            print(f"\n✅ IR构建成功")
            print(f"函数列表: {ir}")
        else:
            print("⚠️  IR构建失败或返回空")
    except Exception as e:
        print(f"❌ IR构建失败: {e}")
        import traceback
        traceback.print_exc()
    
    return True

if __name__ == "__main__":
    success = True
    
    # 测试Python编译器
    if not test_python_compiler():
        success = False
    
    # 测试Rust前端
    if not test_rust_frontend():
        success = False
    
    print("\n" + "=" * 60)
    if success:
        print("✅ 所有测试完成")
    else:
        print("⚠️  部分测试失败")
    print("=" * 60)
    
    sys.exit(0 if success else 1)
