#!/usr/bin/env python3
import sys
import ctypes
sys.path.append('../compiler')

from ffi import ffi

print(f"1. Frontend loaded: {ffi.has_frontend()}")
print(f"2. Version: {ffi.version()}")

# 直接测试FFI
if ffi.has_frontend():
    code = b"fn main():\n    print(123)"
    print(f"3. Calling with {len(code)} bytes")
    
    # 创建缓冲区
    buf = (ctypes.c_ubyte * len(code)).from_buffer_copy(code)
    print(f"4. Buffer created")
    
    # 调用
    print(f"5. Calling chim_lex...")
    result_ptr = ffi.frontend.chim_lex(buf, len(code))
    print(f"6. Got pointer: {hex(result_ptr) if result_ptr else 'NULL'}")
    
    if result_ptr:
        # 转换为c_char_p
        char_p = ctypes.cast(result_ptr, ctypes.c_char_p)
        print(f"7. Cast to c_char_p")
        
        # 获取值
        print(f"8. Getting value...")
        value = char_p.value
        print(f"9. Got bytes: {len(value) if value else 0}")
        
        if value:
            text = value.decode('utf-8')
            print(f"10. Decoded: {len(text)} chars")
            print(f"\n结果前100字符:\n{text[:100]}")
        
        # 释放
        print(f"\n11. Freeing...")
        ffi.frontend.chim_free_string(result_ptr)
        print(f"12. Done!")
