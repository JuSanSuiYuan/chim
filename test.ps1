# Chim 测试脚本 (Windows)

Write-Host "正在运行所有测试..." -ForegroundColor Cyan
cargo test --workspace

Write-Host "✓ 测试完成" -ForegroundColor Green
