# Chim 构建脚本 (Windows)

Write-Host "正在构建 Chim 编译器..." -ForegroundColor Cyan
cargo build --workspace --release

Write-Host "✓ 构建完成" -ForegroundColor Green
Write-Host "可执行文件位于: target\release\chim.exe" -ForegroundColor Yellow
