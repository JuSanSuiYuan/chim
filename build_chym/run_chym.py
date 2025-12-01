import sys
import os
sys.path.insert(0, os.path.dirname(__file__))
from importlib import import_module

# 入口模块：chym.内核入口
mod = import_module('chym.内核入口')
if hasattr(mod, '主'):
    mod.主()