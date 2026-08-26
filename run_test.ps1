$proc = Start-Process "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\qemu\qemu-system-x86_64.exe" -ArgumentList "-drive format=raw,file=`"C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os\target\x86_64-catalyst\debug\catalyst-kernel.img`" -m 512M -display none -serial stdio -no-reboot" -NoNewWindow -RedirectStandardOutput .\qout_live.txt -PassThru
Start-Sleep -Seconds 3
Get-Content .\qout_live.txt
Stop-Process -Id $proc.Id -Force
