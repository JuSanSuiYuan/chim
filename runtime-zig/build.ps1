$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path (Join-Path $PSScriptRoot 'zig-out\lib') | Out-Null
zig build-lib (Join-Path $PSScriptRoot 'src\lib.zig') -dynamic -O ReleaseSafe -lc -femit-bin (Join-Path $PSScriptRoot 'zig-out\lib\chim_backend.dll')
