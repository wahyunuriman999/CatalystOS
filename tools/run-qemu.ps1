# ==========================================
# AEGIS COGNITIVE RUNTIME PLATFORM
# PROPRIETARY AND CONFIDENTIAL
# Copyright (c) 2024-2026 Wahyu Nur Iman. 
# All rights reserved.
# ==========================================

# Catalyst OS — QEMU Launch Script (M1)
# Usage: powershell -File tools/run-qemu.ps1 [path-to-image]

param(
    [string]$ImagePath = "target\x86_64-catalyst\debug\catalyst-kernel.img"
)

# Find QEMU
$qemuPaths = @(
    "C:\Program Files\qemu\qemu-system-x86_64.exe",
    "C:\Program Files (x86)\qemu\qemu-system-x86_64.exe",
    "$env:ProgramFiles\qemu\qemu-system-x86_64.exe"
)

$qemu = $null
foreach ($p in $qemuPaths) {
    if (Test-Path $p) {
        $qemu = $p
        break
    }
}

# Also check PATH
if (-not $qemu) {
    $qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source
}

if (-not $qemu) {
    Write-Error "QEMU not found. Install with: winget install SoftwareFreedomConservancy.QEMU"
    exit 1
}

if (-not (Test-Path $ImagePath)) {
    Write-Error "Boot image not found: $ImagePath"
    Write-Error "Build first with: cargo build -p catalyst-kernel"
    exit 1
}

Write-Host "======================================="
Write-Host "  CATALYST OS — QEMU Boot (M1)"
Write-Host "  Image: $ImagePath"
Write-Host "  QEMU: $qemu"
Write-Host "======================================="
Write-Host ""

# Launch QEMU with:
# - UEFI firmware (OVMF) via built-in QEMU UEFI support
# - 256MB RAM (minimal for M1)
# - Serial output to console (stdio)
# - No graphics window (-nographic) — serial only for M1
& $qemu `
    -drive "format=raw,file=$ImagePath" `
    -m 256M `
    -serial stdio `
    -nographic `
    -no-reboot `
    -no-shutdown
