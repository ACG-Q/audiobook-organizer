param(
    [switch]$Release = $true
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "=== 构建所有 CLI 工具 + 上位机 ===" -ForegroundColor Cyan

# 1. 构建所有 Rust 工作区成员（CLI 工具 + host 后端）
Write-Host "`n[1/2] 编译 Rust 工作区..." -ForegroundColor Yellow
if ($Release) {
    cargo build --release
} else {
    cargo build
}
if ($LASTEXITCODE -ne 0) { throw "Rust 编译失败" }

# 2. 打包 Tauri 上位机
Write-Host "`n[2/2] 打包 Tauri 上位机..." -ForegroundColor Yellow
Push-Location (Join-Path $root "host")
try {
    if ($Release) {
        cargo tauri build
    } else {
        cargo tauri dev
    }
    if ($LASTEXITCODE -ne 0) { throw "Tauri 打包失败" }
} finally {
    Pop-Location
}

Write-Host "`n=== 全部完成 ===" -ForegroundColor Green
