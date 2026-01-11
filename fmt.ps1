# Chim 格式化脚本 (Windows)

Write-Host "正在格式化所有 Rust 代码..." -ForegroundColor Cyan
cargo fmt --all

Write-Host "✓ 格式化完成" -ForegroundColor Green
