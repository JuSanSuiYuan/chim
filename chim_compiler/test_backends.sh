#!/bin/bash
# 测试所有后端

echo "=== Chim编译器多后端测试 ==="
echo ""

test_file="../tests/value_type_test.chim"
backends=("wasm" "native" "llvm" "qbe" "tinycc" "cranelift")

for backend in "${backends[@]}"; do
    echo ">>> 测试后端: $backend"
    ./target/release/chim "$test_file" "$backend" 2>&1 | grep -E "(优化|RVO|使用后端|生成代码成功)"
    echo ""
done

echo "=== 生成的文件 ==="
ls -lh ../tests/value_type_test.* | grep -v ".chim$"
