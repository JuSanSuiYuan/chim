#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
启语(Chim)测试运行脚本
用于编译并运行启语示例程序
"""

import os
import sys
import subprocess

# 添加编译器目录到Python路径
COMPILER_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'compiler'))
sys.path.append(COMPILER_DIR)

def run_example(file_path):
    """运行指定的启语示例文件"""
    print(f"\n======= 运行示例: {file_path} =======")
    try:
        # 导入编译器主程序
        from main import compile_file
        
        # 编译文件
        python_code = compile_file(file_path)
        
        # 打印生成的Python代码
        print("\n生成的Python代码:")
        print("-" * 50)
        print(python_code)
        print("-" * 50)
        
        # 执行生成的Python代码
        print("\n执行结果:")
        print("=" * 50)
        exec(python_code)
        print("=" * 50)
        print("✓ 执行成功")
        
        return True
    except Exception as e:
        print(f"✗ 执行失败: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """主函数"""
    # 示例文件目录
    examples_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'examples'))
    
    # 获取所有.chim文件
    chim_files = [f for f in os.listdir(examples_dir) if f.endswith('.chim')]
    
    if not chim_files:
        print("没有找到.chim示例文件")
        return 1
    
    print(f"找到 {len(chim_files)} 个示例文件:")
    for f in chim_files:
        print(f"  - {f}")
    
    # 运行所有示例
    success_count = 0
    for chim_file in chim_files:
        file_path = os.path.join(examples_dir, chim_file)
        if run_example(file_path):
            success_count += 1
    
    # 打印汇总结果
    print(f"\n======= 测试结果 =======")
    print(f"成功: {success_count}/{len(chim_files)}")
    
    return 0 if success_count == len(chim_files) else 1

if __name__ == "__main__":
    sys.exit(main())