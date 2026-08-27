# Test boot output capture
$proc = Start-Process -FilePath "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\qemu\qemu-system-x86_64.exe" -ArgumentList "-drive format=raw,file=target\x86_64-catalyst\debug\catalyst-kernel.img -m 256M -serial file:target\serial_out.txt -display none -no-reboot" -PassThru
Start-Sleep -Seconds 4
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
if (Test-Path "target\serial_out.txt") {
    Get-Content "target\serial_out.txt" | Select-Object -First 50
}
