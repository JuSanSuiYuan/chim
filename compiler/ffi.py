import os
import ctypes
from ctypes import c_uint, c_ulonglong, c_char_p, c_void_p
from pathlib import Path

class ChimIRArg(ctypes.Structure):
    _fields_ = [("name", ctypes.c_char_p), ("type", ctypes.c_char_p)]

class ChimIRFunction(ctypes.Structure):
    _fields_ = [("name", ctypes.c_char_p), ("args", ctypes.POINTER(ChimIRArg)), ("arg_count", ctypes.c_uint), ("return_type", ctypes.c_char_p)]

class ChimIRModule(ctypes.Structure):
    _fields_ = [("funcs", ctypes.POINTER(ChimIRFunction)), ("func_count", ctypes.c_uint)]

class ChimFFI:
    def __init__(self):
        self.frontend = None
        self.backend = None
        self._load()

    def _load(self):
        base = Path(__file__).parents[1]
        fe_candidates = [
            base / 'compiler-rs' / 'target' / 'release' / ('chim_frontend.dll'),
            base / 'compiler-rs' / 'target' / 'debug' / ('chim_frontend.dll'),
        ]
        for p in fe_candidates:
            if p.exists():
                try:
                    self.frontend = ctypes.CDLL(str(p))
                    # 设置函数签名
                    self.frontend.chim_version.restype = c_uint
                    self.frontend.chim_lex.argtypes = [ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t]
                    self.frontend.chim_lex.restype = ctypes.POINTER(ctypes.c_char)  # 返回原始指针
                    self.frontend.chim_free_string.argtypes = [ctypes.POINTER(ctypes.c_char)]
                    self.frontend.chim_free_string.restype = None
                    self.frontend.chim_build_ir.argtypes = [ctypes.POINTER(ctypes.c_ubyte), c_ulonglong, ctypes.POINTER(ChimIRModule)]
                    self.frontend.chim_build_ir.restype = c_uint
                    self.frontend.chim_ir_free.argtypes = [ctypes.POINTER(ChimIRModule)]
                    self.frontend.chim_ir_free.restype = None
                    break
                except OSError as e:
                    # 可能是架构不匹配 (32bit Python vs 64bit DLL)
                    print(f"Warning: 无法加载 {p.name}: {e}")
                    self.frontend = None

        be_candidates = [
            base / 'runtime-zig' / 'zig-out' / 'lib' / ('chim_backend.dll'),
        ]
        for p in be_candidates:
            if p.exists():
                try:
                    self.backend = ctypes.CDLL(str(p))
                    self.backend.chim_backend_codegen.argtypes = [ctypes.POINTER(ChimIRModule)]
                    self.backend.chim_backend_codegen.restype = c_uint
                    self.backend.chim_send.argtypes = [c_void_p, ctypes.c_longlong]
                    self.backend.chim_send.restype = None
                    self.backend.chim_recv.argtypes = [c_void_p]
                    self.backend.chim_recv.restype = ctypes.c_longlong
                except OSError:
                    self.backend = None

    def has_frontend(self):
        return self.frontend is not None

    def version(self):
        if not self.frontend:
            return 0
        return self.frontend.chim_version()
    
    def lex(self, source: bytes):
        """Rust词法分析"""
        if not self.frontend:
            return None
        buf = (ctypes.c_ubyte * len(source)).from_buffer_copy(source)
        result_ptr = self.frontend.chim_lex(buf, len(source))
        if result_ptr:
            # result_ptr现在是POINTER(c_char)
            result_str = ctypes.cast(result_ptr, ctypes.c_char_p).value
            if result_str:
                result_str = result_str.decode('utf-8')
            self.frontend.chim_free_string(result_ptr)
            return result_str
        return None

    def build_ir(self, source: bytes):
        if not self.frontend:
            return None
        mod = ChimIRModule()
        buf = (ctypes.c_ubyte * len(source)).from_buffer_copy(source)
        ok = self.frontend.chim_build_ir(buf, c_ulonglong(len(source)), ctypes.byref(mod))
        if ok != 1:
            return None
        # Extract IR
        funcs = []
        for i in range(mod.func_count):
            f = mod.funcs[i]
            name = f.name.decode('utf-8') if f.name else ''
            # 仅返回名字，详细参数由 build_ir_module 使用
            funcs.append(name)
        # Free
        self.frontend.chim_ir_free(ctypes.byref(mod))
        return {"functions": funcs}

    def build_ir_module(self, source: bytes):
        if not self.frontend:
            return None
        mod = ChimIRModule()
        buf = (ctypes.c_ubyte * len(source)).from_buffer_copy(source)
        ok = self.frontend.chim_build_ir(buf, c_ulonglong(len(source)), ctypes.byref(mod))
        if ok != 1:
            return None
        return mod

    def free_ir(self, mod: ChimIRModule):
        if self.frontend:
            self.frontend.chim_ir_free(ctypes.byref(mod))

    def has_backend(self):
        return self.backend is not None

    def backend_codegen(self, ir_module: ChimIRModule) -> bool:
        if not self.backend:
            return False
        ok = self.backend.chim_backend_codegen(ctypes.byref(ir_module))
        return ok == 1

    def send(self, ch, value: int):
        if self.backend:
            self.backend.chim_send(c_void_p(id(ch)), ctypes.c_longlong(value))

    def recv(self, ch):
        if self.backend:
            return int(self.backend.chim_recv(c_void_p(id(ch))))
        return 0

ffi = ChimFFI()
