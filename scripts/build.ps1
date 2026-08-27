# ==========================================
# AEGIS COGNITIVE RUNTIME PLATFORM
# PROPRIETARY AND CONFIDENTIAL
# Copyright (c) 2024-2026 Wahyu Nur Iman.
# All rights reserved.
# ==========================================

$ErrorActionPreference = "Stop"
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  CatalystOS Deterministic Build System  " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

$WorkspaceRoot = (Get-Item $PSScriptRoot).Parent.FullName

# 1. Build Userland Applications
Write-Host "`n[1/3] Building Userland Programs (hello, sh, sessiond, displayd, inputd, demo_app, cpkg)..." -ForegroundColor Yellow

$UserlandProjects = @("hello", "sh", "sessiond", "displayd", "inputd", "demo_app", "cpkg", "objectd", "workspaced", "clipboardd", "terminal", "antigravity", "chrome")
foreach ($proj in $UserlandProjects) {
    Write-Host "  -> Building userland/$proj..."
    Push-Location "$WorkspaceRoot\userland\$proj"
    try {
        cargo build --target x86_64-unknown-none
    } finally {
        Pop-Location
    }
}

# 2. Build Catalyst Microkernel
Write-Host "`n[2/3] Building Catalyst Microkernel..." -ForegroundColor Yellow
Push-Location "$WorkspaceRoot"
try {
    cargo build -p catalyst-kernel --target "$WorkspaceRoot\x86_64-catalyst.json"
} finally {
    Pop-Location
}

# 3. Create Bootable Disk Image
Write-Host "`n[3/3] Generating Bootable BIOS Disk Image..." -ForegroundColor Yellow
if (Test-Path "$WorkspaceRoot\..\catalyst-mkimage") {
    Push-Location "$WorkspaceRoot\..\catalyst-mkimage"
    try {
        cargo run
    } finally {
        Pop-Location
    }
} elseif (Test-Path "$WorkspaceRoot\tools\mkimage") {
    Push-Location "$WorkspaceRoot\tools\mkimage"
    try {
        cargo run --target x86_64-pc-windows-msvc
    } finally {
        Pop-Location
    }
}

Write-Host "`n>>> CatalystOS Build Complete: target/x86_64-catalyst/debug/catalyst-kernel.img" -ForegroundColor Green
