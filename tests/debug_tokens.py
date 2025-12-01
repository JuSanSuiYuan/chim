#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
简单调试测试
"""
import sys
sys.path.append('../compiler')

from lexer import Lexer, TokenType

source = """system

fn test():
    令 snapshot := 快照 data
"""

lexer = Lexer(source)
tokens = lexer.tokenize()

print("Tokens:")
for i, tok in enumerate(tokens):
    print(f"{i}: {tok}")
    if i > 30:
        break
