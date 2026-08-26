# ==========================================
# AEGIS COGNITIVE RUNTIME PLATFORM
# PROPRIETARY AND CONFIDENTIAL
# Copyright (c) 2024-2026 Wahyu Nur Iman.
# All rights reserved.
# ==========================================

# Catalyst OS — Build & Run Script
# Usage: powershell -File tools/build-and-run.ps1 [-Release] [-NoBoot]

param(
    [switch]$Release,
    [switch]$NoBoot
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not $root) { $root = (Get-Location).Path }
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

Write-Host "=======================================" -ForegroundColor Cyan
Write-Host "  CATALYST OS — Build System" -ForegroundColor Cyan
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host ""

# Phase 1: Build kernel for bare-metal custom target
Write-Host "[1/3] Building Catalyst Kernel..." -ForegroundColor Yellow
$profile = if ($Release) { "--release" } else { "" }
$profileDir = if ($Release) { "release" } else { "debug" }

$buildArgs = @("build", "-p", "catalyst-kernel")
if ($Release) { $buildArgs += "--release" }

& cargo @buildArgs
if ($LASTEXITCODE -ne 0) {
    Write-Host "KERNEL BUILD FAILED" -ForegroundColor Red
    exit 1
}

$kernelBin = Join-Path $root "target\x86_64-catalyst\$profileDir\catalyst-kernel"
if (-not (Test-Path $kernelBin)) {
    Write-Host "Kernel binary not found at: $kernelBin" -ForegroundColor Red
    exit 1
}
$kernelSize = [math]::Round((Get-Item $kernelBin).Length / 1024, 1)
Write-Host "  Kernel binary: $kernelSize KB" -ForegroundColor Green

# Phase 2: Create UEFI boot image using bootloader crate's runner
Write-Host ""
Write-Host "[2/3] Creating UEFI boot image..." -ForegroundColor Yellow

# Use bootloader crate's built-in disk image creation
# The bootloader crate v0.11 provides a bootloader-runner binary
$imgPath = "$kernelBin.img"

# Try using bootloader_disk_image tool
& cargo install bootimage --quiet 2>$null

# Alternative: use the bootloader crate's create-disk-image feature directly
# We'll use a simpler approach - call cargo bootimage or use bootloader runner
$bootloaderRunner = & cargo metadata --format-version 1 2>$null | ConvertFrom-Json | 
    Select-Object -ExpandProperty packages | 
    Where-Object { $_.name -eq "bootloader" } |
    Select-Object -ExpandProperty manifest_path -ErrorAction SilentlyContinue

# Direct approach: use bootloader's disk image creation
# For bootloader v0.11, we need to use its built-in UEFI boot disk creation
Write-Host "  Using bootloader crate runner..." -ForegroundColor Gray

# The bootloader v0.11 runner creates disk images automatically when used as cargo runner
# Let's use cargo run which triggers the runner
& cargo run -p catalyst-kernel $profile 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  Note: Runner may need QEMU. Trying direct image creation..." -ForegroundColor Yellow
}

if (Test-Path $imgPath) {
    $imgSize = [math]::Round((Get-Item $imgPath).Length / (1024*1024), 1)
    Write-Host "  Boot image: $imgSize MB" -ForegroundColor Green
} else {
    Write-Host "  Boot image not found. QEMU boot will use kernel binary directly." -ForegroundColor Yellow
}

if ($NoBoot) {
    Write-Host ""
    Write-Host "Build complete. Skipping QEMU boot (-NoBoot)." -ForegroundColor Green
    exit 0
}

# Phase 3: Boot in QEMU
Write-Host ""
Write-Host "[3/3] Booting in QEMU..." -ForegroundColor Yellow
& powershell -File "$root\tools\run-qemu.ps1" -ImagePath $imgPath
