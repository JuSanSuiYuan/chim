#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parents[1] / 'compiler'))
from lexer import Lexer

if __name__ == '__main__':
    src = Path(sys.argv[1])
    code = src.read_text(encoding='utf-8')
    lex = Lexer(code)
    toks = lex.tokenize()
    for t in toks:
        print(t)
