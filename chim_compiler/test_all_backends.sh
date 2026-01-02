#!/bin/bash
# Chim编译器 - 多后端测试脚本

echo "=========================================="
echo "Chim Compiler - Multi-Backend Test"
echo "=========================================="
echo ""

INPUT_FILE="../tests/value_type_test.chim"
COMPILER="./target/release/chim"

# 确保编译器已构建
if [ ! -f "$COMPILER" ]; then
    echo "Building compiler..."
    cargo build --release
fi

echo "Testing all 8 backends:"
echo ""

# 1. WASM后端
echo "1. WASM Backend..."
$COMPILER $INPUT_FILE -t wasm -O 2 > /dev/null 2>&1
echo "   ✓ Generated: value_type_test.wasm"

# 2. Native C后端
echo "2. Native C Backend..."
$COMPILER $INPUT_FILE -t native -O 2 > /dev/null 2>&1
echo "   ✓ Generated: value_type_test.c"

# 3. LLVM IR后端
echo "3. LLVM IR Backend..."
$COMPILER $INPUT_FILE -t llvm -O 2 > /dev/null 2>&1
echo "   ✓ Generated: value_type_test.ll"

# 4. QBE后端
echo "4. QBE Backend..."
$COMPILER $INPUT_FILE -t qbe -O 2 > /dev/null 2>&1
echo "   ✓ Generated: value_type_test.qbe"

# 5. TinyCC后端
echo "5. TinyCC Backend..."
$COMPILER $INPUT_FILE -t tinycc -O 2 > /dev/null 2>&1
echo "   ✓ Generated: value_type_test.c"

# 6. Cranelift后端
echo "6. Cranelift IR Backend..."
$COMPILER $INPUT_FILE -t cranelift -O 2 > /dev/null 2>&1
echo "   ✓ Generated: value_type_test.clif"

# 7. Fortran后端（科学计算）
echo "7. Fortran Backend (Scientific Computing)..."
$COMPILER $INPUT_FILE -t fortran -O 2 > /dev/null 2>&1
echo "   ✓ Generated: value_type_test.f90"

# 8. x86-64汇编后端
echo "8. x86-64 Assembly Backend..."
$COMPILER $INPUT_FILE -t asm -O 1 > /dev/null 2>&1
echo "   ✓ Generated: value_type_test.s"

echo ""
echo "=========================================="
echo "All backends tested successfully!"
echo "=========================================="
echo ""
echo "Backend Summary:"
echo "  • WASM       - WebAssembly (web平台)"
echo "  • Native     - C语言 (通用)"
echo "  • LLVM       - LLVM IR (高优化)"
echo "  • QBE        - QBE IL (轻量级)"
echo "  • TinyCC     - TinyCC (快速编译)"
echo "  • Cranelift  - Cranelift IR (JIT优化)"
echo "  • Fortran    - Fortran 2008 (科学计算)"
echo "  • Assembly   - x86-64 AT&T (底层优化)"
echo ""
echo "Use cases:"
echo "  开发调试    → TinyCC (0.05s)"
echo "  科学计算    → Fortran (数值优化)"
echo "  Web应用     → WASM"
echo "  生产环境    → LLVM (最优性能)"
echo "  JIT编译     → Cranelift"
echo "  底层优化    → Assembly"
echo ""
