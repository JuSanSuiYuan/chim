#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'compiler'))

from lexer import Lexer

test_code = """
system

fn 测试解引用语法() -> 整数:
    设 a := 100
    设 pa := &a
    设 v1 := pa.*
    设 pb := &a
    设 v2 := *pb
    返回 v1 + v2
"""

lexer = Lexer(test_code)
tokens = lexer.tokenize()

print("查看第42行附近的tokens:")
for i, tok in enumerate(tokens):
    if i >= 10 and i <= 30:
        print(f"{i}: Line {tok.line}, Col {tok.column} -> {tok}")
