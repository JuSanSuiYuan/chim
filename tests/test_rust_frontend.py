#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
sys.path.append('../compiler')

from ffi import ffi

print(f"Rust前端已加载: {ffi.has_frontend()}")
if ffi.has_frontend():
    print(f"版本: {ffi.version()}")
    
    # 简单测试
    code = "fn 主():\n    输出(\"hello\")"
    print(f"\n测试代码: {code}")
    
    result = ffi.lex(code.encode('utf-8'))
    print(f"\n结果:\n{result}")
