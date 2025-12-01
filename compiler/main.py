#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
启语(Chim)编译器 - 主入口
"""

import sys
import os
from pathlib import Path

# 添加父目录到路径，使模块可以相互导入
sys.path.insert(0, str(Path(__file__).parent))

from lexer import Lexer
from parser_ import Parser
from codegen import CodeGenerator, generate_code
from semantic import SemanticAnalyzer
from zig_codegen import generate_zig_code
from ffi import ffi

def compile_file(source_file: str, backend: str = "python", verbose: bool = False) -> str:
    """
    编译文件
    backend: "python" 或 "zig"
    verbose: 是否显示详细信息
    """
    p = Path(source_file)
    
    if verbose:
        print(f"\n[编译] {source_file}")
        print("=" * 60)
    
    with open(p, 'r', encoding='utf-8') as f:
        source_code = f.read()
    
    if verbose:
        print(f"\n[1/4] 词法分析...")
    
    # Rust前端尝试
    if ffi.has_frontend():
        src_bytes = source_code.encode('utf-8')
        ir = ffi.build_ir(src_bytes)
        if ir:
            if verbose:
                print(f"  ✓ Rust 前端 IR: {ir}")
            use_ir = os.environ.get('CHIM_USE_IR') == '1'
            if use_ir:
                mod = ffi.build_ir_module(src_bytes)
                if mod:
                    try:
                        from codegen import generate_code_from_ir_full
                        py_code = generate_code_from_ir_full(mod)
                        return py_code
                    finally:
                        ffi.free_ir(mod)
                else:
                    from codegen import generate_code_from_ir
                    py_code = generate_code_from_ir(ir.get('functions', []))
                    return py_code
            if ffi.has_backend():
                mod = ffi.build_ir_module(src_bytes)
                if mod:
                    ok = ffi.backend_codegen(mod)
                    ffi.free_ir(mod)
                    if verbose:
                        print(f"  Zig 后端: {'OK' if ok else 'FAIL'}")
    
    # Python前端
    try:
        lexer = Lexer(source_code)
        tokens = lexer.tokenize()
        if verbose:
            print(f"  ✓ {len(tokens)} tokens")
    except Exception as e:
        print(f"\n✗ 词法错误: {e}")
        raise
    
    if verbose:
        print(f"\n[2/4] 语法分析...")
    
    try:
        parser = Parser(tokens)
        ast = parser.parse()
        dialect = parser.dialect
        if verbose:
            print(f"  ✓ 方言: {dialect}")
            print(f"  ✓ {len(ast.statements)} 顶级语句")
    except Exception as e:
        print(f"\n✗ 语法错误: {e}")
        raise
    
    if verbose:
        print(f"\n[3/4] 语义分析...")
    
    # 语义分析
    analyzer = SemanticAnalyzer(dialect=dialect)
    if not analyzer.analyze(ast):
        errors = analyzer.get_errors()
        print(f"\n✗ 语义错误: 发现 {len(errors)} 个错误")
        for i, err in enumerate(errors, 1):
            print(f"  {i}. {err}")
        if not verbose:
            print("\n警告: 存在语义错误，但继续生成代码...\n")
    else:
        if verbose:
            print(f"  ✓ 语义检查通过")
            # 显示生命周期信息
            if analyzer.lifetime_analyzer.group_vars:
                print(f"  ✓ 组块: {list(analyzer.lifetime_analyzer.group_vars.keys())}")
    
    if verbose:
        print(f"\n[4/4] 代码生成 ({backend})...")
    
    # 选择后端
    try:
        if backend == "zig":
            code = generate_zig_code(ast, dialect=dialect)
        else:
            code = generate_code(ast, dialect=dialect)
        
        if verbose:
            print(f"  ✓ 生成 {len(code.splitlines())} 行代码")
        
        return code
    except Exception as e:
        print(f"\n✗ 代码生成错误: {e}")
        raise

def main():
    if len(sys.argv) < 2:
        print("用法: chim <源文件.chim> [--backend python|zig] [--verbose]")
        print("\n选项:")
        print("  --backend python|zig  选择后端 (默认: python)")
        print("  --verbose             显示详细编译信息")
        print("\n示例:")
        print("  chim hello.chim")
        print("  chim test.chim --verbose")
        print("  chim kernel.chim --backend zig")
        sys.exit(1)
    
    source_file = sys.argv[1]
    backend = "python"  # 默认Python后端
    verbose = "--verbose" in sys.argv or "-v" in sys.argv
    
    # 解析参数
    if "--backend" in sys.argv:
        idx = sys.argv.index("--backend")
        if idx + 1 < len(sys.argv):
            backend = sys.argv[idx + 1]
    
    if not source_file.endswith('.chim'):
        print(f"错误: {source_file} 不是 .chim 文件")
        sys.exit(1)
    
    try:
        code = compile_file(source_file, backend=backend, verbose=verbose)
    except FileNotFoundError:
        print(f"错误: 找不到文件 {source_file}")
        sys.exit(1)
    except Exception as e:
        print(f"\n编译失败: {e}")
        sys.exit(1)
    
    out_dir = Path("d:/PROJECT/Chim/build")
    out_dir.mkdir(parents=True, exist_ok=True)
    
    # 根据后端选择扩展名
    ext = ".zig" if backend == "zig" else ".py"
    out_file = out_dir / (Path(source_file).stem + ext)
    
    with open(out_file, "w", encoding="utf-8") as fh:
        fh.write(code)
    
    if verbose:
        print(f"\n=" * 60)
    print(f"✓ 生成: {out_file}")
    
    if backend == "python":
        print(f"✓ 运行: python {out_file}")
    elif backend == "zig":
        print(f"✓ 运行: zig run {out_file}")

if __name__ == "__main__":
    main()
