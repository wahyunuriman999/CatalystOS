# ==========================================
# CATALYST OS — PHASE H1 FLASH SCRIPT
# TARGET: DISK 1 (SANDISK USB ONLY)
# ==========================================
$ErrorActionPreference = "Stop"
$LogPath = "$PSScriptRoot\..\target\flash_result.log"

function Log-Msg($msg) {
    Write-Host $msg
    Add-Content -Path $LogPath -Value $msg
}

Set-Content -Path $LogPath -Value "=== CATALYST OS FLASH LOG ==="

$ImagePath = "$PSScriptRoot\..\target\x86_64-catalyst\debug\catalyst-kernel.img"
$ExpectedSha256 = "298587EF635AB5C67429F55F7AACB1A29500FB9F727B0BC18F69299EC6CD3CAC"
$ExpectedSizeBytes = 10978304

Log-Msg ">>> [1/5] Verifying Source Binary Integrity..."
if (-not (Test-Path $ImagePath)) {
    Log-Msg "ERROR: Source image not found at $ImagePath"
    exit 1
}

$FileInfo = Get-Item $ImagePath
if ($FileInfo.Length -ne $ExpectedSizeBytes) {
    Log-Msg "ERROR: Size mismatch! Expected $ExpectedSizeBytes bytes, got $($FileInfo.Length) bytes"
    exit 1
}

$ActualSha256 = (Get-FileHash -Algorithm SHA256 $ImagePath).Hash
if ($ActualSha256 -ne $ExpectedSha256) {
    Log-Msg "ERROR: SHA-256 MISMATCH! Expected $ExpectedSha256, got $ActualSha256. ABORTING."
    exit 1
}
Log-Msg "  -> Source SHA-256 Verified: $ActualSha256"

Log-Msg "`n>>> [2/5] Performing Strict Physical Disk Safety Audit..."
$Disk0 = Get-Disk -Number 0
if ($Disk0.FriendlyName -notmatch "INTEL") {
    Log-Msg "SAFETY ABORT: Disk 0 identity unexpected!"
    exit 1
}
Log-Msg "  -> Disk 0 (Protected System SSD): $($Disk0.FriendlyName) - LOCKED & PROTECTED"

$Disk1 = Get-Disk -Number 1
if ($Disk1.FriendlyName -notmatch "SanDisk" -or $Disk1.BusType -ne "USB" -or $Disk1.SerialNumber -notmatch "04016627112524140951") {
    Log-Msg "SAFETY ABORT: Target Disk 1 does not match SanDisk USB device criteria!"
    exit 1
}
Log-Msg "  -> Disk 1 Target Verified: $($Disk1.FriendlyName) (Bus: $($Disk1.BusType), Serial: $($Disk1.SerialNumber))"

Log-Msg "`n>>> [3/5] Dismounting and Clearing Volume E: locks..."
try {
    & fsutil volume dismount E: | Out-Null
} catch {}

Log-Msg "`n>>> [4/5] Writing Raw Binary Image to \\.\PhysicalDrive1..."
$ImageBytes = [System.IO.File]::ReadAllBytes((Resolve-Path $ImagePath).Path)
Log-Msg "  -> Loaded $($ImageBytes.Length) bytes from source image."

$PhysicalDrivePath = "\\.\PhysicalDrive1"
try {
    $driveHandle = [System.IO.File]::Open($PhysicalDrivePath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::ReadWrite)
    $driveHandle.Seek(0, [System.IO.SeekOrigin]::Begin) | Out-Null
    $driveHandle.Write($ImageBytes, 0, $ImageBytes.Length)
    $driveHandle.Flush($true)
    $driveHandle.Close()
    $driveHandle.Dispose()
    Log-Msg "  -> Wrote $($ImageBytes.Length) bytes to PhysicalDrive1 successfully!"
} catch {
    Log-Msg "ERROR writing to PhysicalDrive1: $_"
    exit 1
}

Log-Msg "`n>>> [5/5] Verifying Written Image Integrity on Disk 1..."
try {
    $verifyHandle = [System.IO.File]::Open($PhysicalDrivePath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
    $readBuffer = New-Object byte[] $ExpectedSizeBytes
    $bytesRead = $verifyHandle.Read($readBuffer, 0, $ExpectedSizeBytes)
    $verifyHandle.Close()
    $verifyHandle.Dispose()
    
    if ($bytesRead -ne $ExpectedSizeBytes) {
        Log-Msg "ERROR: Verification read incomplete! Read $bytesRead of $ExpectedSizeBytes bytes."
        exit 1
    }
    
    $sha256Managed = [System.Security.Cryptography.SHA256]::Create()
    $targetHashBytes = $sha256Managed.ComputeHash($readBuffer)
    $targetHash = ($targetHashBytes | ForEach-Object { $_.ToString("X2") }) -join ""
    
    if ($targetHash -ne $ExpectedSha256) {
        Log-Msg "ERROR: Written hash ($targetHash) does not match expected ($ExpectedSha256)!"
        exit 1
    }
    Log-Msg "  -> Target Disk 1 SHA-256 Verified: $targetHash (EXACT MATCH!)"
    Log-Msg "`n>>> FLASH COMPLETED & VERIFIED 100% SUCCESSFULLY! <<<"
} catch {
    Log-Msg "ERROR verifying PhysicalDrive1: $_"
    exit 1
}
