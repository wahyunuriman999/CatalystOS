$ImagePath = "target\x86_64-catalyst\debug\catalyst-kernel.img"
$ExpectedSha256 = "298587EF635AB5C67429F55F7AACB1A29500FB9F727B0BC18F69299EC6CD3CAC"
$ExpectedSizeBytes = 10978304

Write-Host "============================================================"
Write-Host "  CATALYST OS — READ-ONLY BOOT IMAGE AUDIT                  "
Write-Host "============================================================"

# 1. File existence & size
$FileItem = Get-Item $ImagePath
Write-Host ""
Write-Host "1. File Size Audit:"
Write-Host "   Expected Size : $ExpectedSizeBytes bytes"
Write-Host "   Actual Size   : $($FileItem.Length) bytes"
$SizePass = ($FileItem.Length -eq $ExpectedSizeBytes)
if ($SizePass) { Write-Host "   Size Status   : PASS" } else { Write-Host "   Size Status   : FAIL" }

# 2. SHA-256 Checksum
$ActualHash = (Get-FileHash -Algorithm SHA256 $ImagePath).Hash
Write-Host ""
Write-Host "2. Checksum Audit:"
Write-Host "   Expected SHA  : $ExpectedSha256"
Write-Host "   Actual SHA    : $ActualHash"
$HashPass = ($ActualHash -eq $ExpectedSha256)
if ($HashPass) { Write-Host "   SHA Status    : PASS" } else { Write-Host "   SHA Status    : FAIL" }

# 3. Read Sector 0 (512 bytes)
$Stream = [System.IO.File]::OpenRead((Resolve-Path $ImagePath).Path)
$Sector0 = New-Object byte[] 512
$null = $Stream.Read($Sector0, 0, 512)
$Stream.Close()

# 4. Boot Signature (Offset 510..511)
$b510 = "{0:X2}" -f $Sector0[510]
$b511 = "{0:X2}" -f $Sector0[511]
Write-Host ""
Write-Host "3. Boot Sector Signature (Offset 0x1FE..0x1FF):"
Write-Host "   Expected Magic: 55 AA (0x55, 0xAA)"
Write-Host "   Actual Magic  : $b510 $b511"
$SigPass = ($Sector0[510] -eq 0x55 -and $Sector0[511] -eq 0xAA)
if ($SigPass) { Write-Host "   Signature     : PASS (Valid BIOS Boot Signature 0xAA55)" } else { Write-Host "   Signature     : FAIL" }

# 5. Bootstrap Entry Point / Opcode inspection (Offset 0..15)
Write-Host ""
Write-Host "4. Entry Point Opcodes (First 16 bytes):"
$First16Hex = ($Sector0[0..15] | ForEach-Object { "{0:X2}" -f $_ }) -join " "
Write-Host "   Hex: $First16Hex"

$b0 = "{0:X2}" -f $Sector0[0]
$b1 = "{0:X2}" -f $Sector0[1]
Write-Host "   Initial Instruction: 0x$b0 0x$b1 (x86 Real Mode Bootstrap Entry)"

# 6. MBR Partition Table Entries (Offset 446..509)
Write-Host ""
Write-Host "5. MBR Partition Table Structure (Offset 0x1BE..0x1FD):"
for ($p = 0; $p -lt 4; $p++) {
    $pOffset = 446 + ($p * 16)
    $pBytes = $Sector0[$pOffset..($pOffset + 15)]
    $pHex = ($pBytes | ForEach-Object { "{0:X2}" -f $_ }) -join " "
    $bootFlag = "{0:X2}" -f $pBytes[0]
    $pType = "{0:X2}" -f $pBytes[4]
    $lbaStart = [BitConverter]::ToUInt32($pBytes, 8)
    $secCount = [BitConverter]::ToUInt32($pBytes, 12)
    $offHex = "{0:X}" -f $pOffset
    Write-Host "   Partition $($p+1) (Offset 0x$offHex): $pHex"
    Write-Host "     Boot Flag : 0x$bootFlag"
    Write-Host "     Part Type : 0x$pType"
    Write-Host "     LBA Start : $lbaStart"
    Write-Host "     Sectors   : $secCount"
}

Write-Host ""
Write-Host "============================================================"
if ($SizePass -and $HashPass -and $SigPass) {
    Write-Host "  AUDIT RESULT: ALL BIOS BOOT INVARIANTS PASS (100% READY)"
} else {
    Write-Host "  AUDIT RESULT: AUDIT FAILED"
}
Write-Host "============================================================"
