#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
import os
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[1] / 'compiler'))
from lexer import Lexer
from parser_ import Parser
from codegen import generate_code

def ensure_pkg_dirs(path):
    path.mkdir(parents=True, exist_ok=True)
    # 为每级目录添加 __init__.py
    cur = path
    while True:
        init_file = cur / '__init__.py'
        if not init_file.exists():
            init_file.write_text("", encoding='utf-8')
        if cur.parent.name == 'build_chym' or cur.parent == cur:
            break
        cur = cur.parent

def compile_file(src_path: Path, dst_root: Path):
    code = src_path.read_text(encoding='utf-8')
    lexer = Lexer(code)
    tokens = lexer.tokenize()
    parser = Parser(tokens)
    ast = parser.parse()
    py_code = generate_code(ast, dialect=parser.dialect)
    rel = src_path.relative_to(src_root)
    out_path = dst_root / rel.with_suffix('.py')
    ensure_pkg_dirs(out_path.parent)
    out_path.write_text(py_code, encoding='utf-8')
    return out_path

if __name__ == '__main__':
    src_root = Path('d:/PROJECT/Chym')
    dst_root = Path('d:/PROJECT/Chim/build_chym') / 'chym'
    dst_root.mkdir(parents=True, exist_ok=True)
    # 根包 __init__.py
    (dst_root / '__init__.py').write_text("", encoding='utf-8')
    generated = []
    for p in src_root.rglob('*.chim'):
        try:
            generated.append(compile_file(p, dst_root))
        except Exception as e:
            print('编译失败:', p, '-', e)
            raise
    print('生成Python文件:')
    for g in generated:
        print(' -', g)
    # 生成运行器
    runner = dst_root.parent / 'run_chym.py'
    runner.write_text(
        """
import sys
import os
sys.path.insert(0, os.path.dirname(__file__))
from importlib import import_module

# 入口模块：chym.内核入口
mod = import_module('chym.内核入口')
if hasattr(mod, '主'):
    mod.主()
""".strip(),
        encoding='utf-8'
    )
    print('运行器:', runner)
    print('使用: python', runner)
