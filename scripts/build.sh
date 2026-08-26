#!/usr/bin/env bash
# ==========================================
# AEGIS COGNITIVE RUNTIME PLATFORM
# PROPRIETARY AND CONFIDENTIAL
# Copyright (c) 2024-2026 Wahyu Nur Iman.
# All rights reserved.
# ==========================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "  CatalystOS Deterministic Build System   "
echo "=========================================="

# 1. Build Userland Programs
echo "[1/3] Building Userland Programs..."
cd "$WORKSPACE_ROOT/userland/hello"
cargo build --target x86_64-catalyst-user.json

# 2. Build Microkernel
echo "[2/3] Building Catalyst Microkernel..."
cd "$WORKSPACE_ROOT/kernel"
cargo build --target x86_64-catalyst.json

# 3. Create Bootable Disk Image
echo "[3/3] Generating Bootable BIOS Disk Image..."
cd "$WORKSPACE_ROOT/tools/mkimage"
cargo run

echo ">>> CatalystOS Build Complete: target/x86_64-catalyst/debug/catalyst-kernel.img"
