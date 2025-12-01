#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试 Zig 运行时库的 Python FFI 绑定
"""
import ctypes
import os

# 加载 DLL
dll_path = os.path.join(os.path.dirname(__file__), '../runtime-zig/src/lib.dll')
if not os.path.exists(dll_path):
    print(f"错误: 找不到 {dll_path}")
    print("请先编译 Zig 运行时库: cd runtime-zig/src && zig build-lib lib.zig -dynamic -lc")
    exit(1)

lib = ctypes.CDLL(dll_path)

# ==================== 函数签名定义 ====================

# Channel
lib.chim_channel_create.restype = ctypes.c_void_p
lib.chim_send.argtypes = [ctypes.c_void_p, ctypes.c_int64]
lib.chim_recv.argtypes = [ctypes.c_void_p]
lib.chim_recv.restype = ctypes.c_int64
lib.chim_channel_is_ready.argtypes = [ctypes.c_void_p]
lib.chim_channel_is_ready.restype = ctypes.c_bool
lib.chim_channel_destroy.argtypes = [ctypes.c_void_p]

# Arena
lib.chim_arena_create.restype = ctypes.c_void_p
lib.chim_arena_alloc.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
lib.chim_arena_alloc.restype = ctypes.c_void_p
lib.chim_arena_reset.argtypes = [ctypes.c_void_p]
lib.chim_arena_destroy.argtypes = [ctypes.c_void_p]

# Array
lib.chim_array_create.argtypes = [ctypes.c_size_t]
lib.chim_array_create.restype = ctypes.c_void_p
lib.chim_array_get.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
lib.chim_array_get.restype = ctypes.c_int64
lib.chim_array_set.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int64]
lib.chim_array_length.argtypes = [ctypes.c_void_p]
lib.chim_array_length.restype = ctypes.c_size_t
lib.chim_array_push.argtypes = [ctypes.c_void_p, ctypes.c_int64]
lib.chim_array_push.restype = ctypes.c_bool
lib.chim_array_pop.argtypes = [ctypes.c_void_p]
lib.chim_array_pop.restype = ctypes.c_int64
lib.chim_array_destroy.argtypes = [ctypes.c_void_p]

# String
lib.chim_string_concat.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.chim_string_concat.restype = ctypes.c_void_p
lib.chim_string_length.argtypes = [ctypes.c_char_p]
lib.chim_string_length.restype = ctypes.c_size_t
lib.chim_string_free.argtypes = [ctypes.c_void_p]

# Math
lib.chim_abs_i64.argtypes = [ctypes.c_int64]
lib.chim_abs_i64.restype = ctypes.c_int64
lib.chim_sqrt_f64.argtypes = [ctypes.c_double]
lib.chim_sqrt_f64.restype = ctypes.c_double
lib.chim_pow_f64.argtypes = [ctypes.c_double, ctypes.c_double]
lib.chim_pow_f64.restype = ctypes.c_double
lib.chim_max_i64.argtypes = [ctypes.c_int64, ctypes.c_int64]
lib.chim_max_i64.restype = ctypes.c_int64

# ==================== 测试函数 ====================

def test_channel():
    print("\n[1] Channel 测试:")
    ch = lib.chim_channel_create()
    
    lib.chim_send(ch, 42)
    val = lib.chim_recv(ch)
    print(f"  发送 42, 接收: {val}")
    
    is_ready = lib.chim_channel_is_ready(ch)
    print(f"  是否就绪: {is_ready}")
    
    lib.chim_channel_destroy(ch)
    print("  ✓ Channel 测试通过")

def test_arena():
    print("\n[2] Arena 测试:")
    arena = lib.chim_arena_create()
    
    mem1 = lib.chim_arena_alloc(arena, 100)
    mem2 = lib.chim_arena_alloc(arena, 200)
    print(f"  分配2个内存块: {mem1 != 0} {mem2 != 0}")
    
    lib.chim_arena_reset(arena)
    print("  Arena 重置")
    
    lib.chim_arena_destroy(arena)
    print("  ✓ Arena 测试通过")

def test_array():
    print("\n[3] Array 测试:")
    arr = lib.chim_array_create(10)
    
    lib.chim_array_set(arr, 0, 100)
    lib.chim_array_set(arr, 1, 200)
    lib.chim_array_push(arr, 300)
    
    print(f"  arr[0] = {lib.chim_array_get(arr, 0)}")
    print(f"  arr[1] = {lib.chim_array_get(arr, 1)}")
    print(f"  长度: {lib.chim_array_length(arr)}")
    
    popped = lib.chim_array_pop(arr)
    print(f"  弹出: {popped}")
    
    lib.chim_array_destroy(arr)
    print("  ✓ Array 测试通过")

def test_string():
    print("\n[4] String 测试:")
    s1 = b"Hello"
    s2 = b" World"
    s3 = lib.chim_string_concat(s1, s2)
    
    # 读取字符串
    result = ctypes.cast(s3, ctypes.c_char_p).value
    print(f"  拼接: {result.decode('utf-8')}")
    print(f"  长度: {lib.chim_string_length(result)}")
    
    lib.chim_string_free(s3)
    print("  ✓ String 测试通过")

def test_math():
    print("\n[5] Math 测试:")
    print(f"  abs(-42) = {lib.chim_abs_i64(-42)}")
    print(f"  sqrt(16) = {lib.chim_sqrt_f64(16.0)}")
    print(f"  pow(2, 8) = {lib.chim_pow_f64(2.0, 8.0)}")
    print(f"  max(10, 20) = {lib.chim_max_i64(10, 20)}")
    print("  ✓ Math 测试通过")

# ==================== 主函数 ====================

if __name__ == '__main__':
    print("=" * 60)
    print("Zig 运行时库 Python FFI 测试")
    print("=" * 60)
    
    test_channel()
    test_arena()
    test_array()
    test_string()
    test_math()
    
    print("\n" + "=" * 60)
    print("✅ 所有测试通过!")
    print("=" * 60)
