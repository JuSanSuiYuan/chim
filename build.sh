#!/bin/bash
# Chim 构建脚本

echo "正在构建 Chim 编译器..."
cargo build --workspace --release

echo "✓ 构建完成"
echo "可执行文件位于: target/release/chim"
