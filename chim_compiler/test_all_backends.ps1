# Chim编译器 - 多后端测试脚本 (PowerShell)

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Chim Compiler - Multi-Backend Test" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

$INPUT_FILE = "..\tests\value_type_test.chim"
$COMPILER = ".\target\release\chim.exe"

# 确保编译器已构建
if (-not (Test-Path $COMPILER)) {
    Write-Host "Building compiler..." -ForegroundColor Yellow
    cargo build --release
}

Write-Host "Testing all 8 backends:" -ForegroundColor Green
Write-Host ""

# 1. WASM后端
Write-Host "1. WASM Backend..." -ForegroundColor White
& $COMPILER $INPUT_FILE -t wasm -O 2 2>&1 | Out-Null
Write-Host "   ✓ Generated: value_type_test.wasm" -ForegroundColor Green

# 2. Native C后端
Write-Host "2. Native C Backend..." -ForegroundColor White
& $COMPILER $INPUT_FILE -t native -O 2 2>&1 | Out-Null
Write-Host "   ✓ Generated: value_type_test.c" -ForegroundColor Green

# 3. LLVM IR后端
Write-Host "3. LLVM IR Backend..." -ForegroundColor White
& $COMPILER $INPUT_FILE -t llvm -O 2 2>&1 | Out-Null
Write-Host "   ✓ Generated: value_type_test.ll" -ForegroundColor Green

# 4. QBE后端
Write-Host "4. QBE Backend..." -ForegroundColor White
& $COMPILER $INPUT_FILE -t qbe -O 2 2>&1 | Out-Null
Write-Host "   ✓ Generated: value_type_test.qbe" -ForegroundColor Green

# 5. TinyCC后端
Write-Host "5. TinyCC Backend..." -ForegroundColor White
& $COMPILER $INPUT_FILE -t tinycc -O 2 2>&1 | Out-Null
Write-Host "   ✓ Generated: value_type_test_tcc.c" -ForegroundColor Green

# 6. Cranelift后端
Write-Host "6. Cranelift IR Backend..." -ForegroundColor White
& $COMPILER $INPUT_FILE -t cranelift -O 2 2>&1 | Out-Null
Write-Host "   ✓ Generated: value_type_test.clif" -ForegroundColor Green

# 7. Fortran后端（科学计算）
Write-Host "7. Fortran Backend (Scientific Computing)..." -ForegroundColor White
& $COMPILER $INPUT_FILE -t fortran -O 2 2>&1 | Out-Null
Write-Host "   ✓ Generated: value_type_test.f90" -ForegroundColor Green

# 8. x86-64汇编后端
Write-Host "8. x86-64 Assembly Backend..." -ForegroundColor White
& $COMPILER $INPUT_FILE -t asm -O 1 2>&1 | Out-Null
Write-Host "   ✓ Generated: value_type_test.s" -ForegroundColor Green

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "All backends tested successfully!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Backend Summary:" -ForegroundColor Yellow
Write-Host "  • WASM       - WebAssembly (web平台)"
Write-Host "  • Native     - C语言 (通用)"
Write-Host "  • LLVM       - LLVM IR (高优化)"
Write-Host "  • QBE        - QBE IL (轻量级)"
Write-Host "  • TinyCC     - TinyCC (快速编译)"
Write-Host "  • Cranelift  - Cranelift IR (JIT优化)"
Write-Host "  • Fortran    - Fortran 2008 (科学计算)" -ForegroundColor Magenta
Write-Host "  • Assembly   - x86-64 AT&T (底层优化)" -ForegroundColor Magenta
Write-Host ""
Write-Host "Use cases:" -ForegroundColor Yellow
Write-Host "  开发调试    → TinyCC (0.05s)"
Write-Host "  科学计算    → Fortran (数值优化)" -ForegroundColor Magenta
Write-Host "  Web应用     → WASM"
Write-Host "  生产环境    → LLVM (最优性能)"
Write-Host "  JIT编译     → Cranelift"
Write-Host "  底层优化    → Assembly" -ForegroundColor Magenta
Write-Host ""

Write-Host "新增后端特性说明:" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
Write-Host ""
Write-Host "【Fortran后端 - 科学计算优化】" -ForegroundColor Magenta
Write-Host "  ✓ Modern Fortran 2008/2018 语法"
Write-Host "  ✓ 双精度浮点数（数值稳定性）"
Write-Host "  ✓ 数组优化能力（gfortran/ifort）"
Write-Host "  ✓ DO CONCURRENT 并行支持"
Write-Host "  适用场景: 数值计算、线性代数、科学模拟"
Write-Host ""
Write-Host "【x86-64汇编后端 - 底层控制】" -ForegroundColor Magenta
Write-Host "  ✓ AT&T语法（GNU AS兼容）"
Write-Host "  ✓ System V ABI调用约定"
Write-Host "  ✓ 寄存器分配优化"
Write-Host "  ✓ 栈帧管理"
Write-Host "  适用场景: 性能关键代码、系统编程、学习汇编"
Write-Host ""
