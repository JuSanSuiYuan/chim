# Chim Runtime Library
# 内置函数实现

try:
    import sys
    sys.path.insert(0, '../compiler')
    from ffi import ffi
    HAS_FFI = True
except Exception as e:
    HAS_FFI = False
    print(f"Warning: 无法加载 chim_frontend.dll: {e}")

def 输出(*args):
    """输出函数 - 支持中文名称"""
    print(*args)

def 范围(start, end):
    """范围函数 - 返回range对象"""
    return range(start, end)

# 英文别名
print_ = 输出

def send(ch, x):
    if HAS_FFI and ffi.has_backend():
        ffi.send(ch, x)
        return None
    return None

def recv(ch):
    if HAS_FFI and ffi.has_backend():
        return ffi.recv(ch)
    return 0
