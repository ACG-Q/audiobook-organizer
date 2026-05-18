#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== 构建所有 CLI 工具 + 上位机 ==="

# 1. 构建所有 Rust 工作区成员
echo ""
echo "[1/2] 编译 Rust 工作区..."
cargo build --release

# 2. 打包 Tauri 上位机
echo ""
echo "[2/2] 打包 Tauri 上位机..."
cd "$ROOT/host"
cargo tauri build

echo ""
echo "=== 全部完成 ==="
