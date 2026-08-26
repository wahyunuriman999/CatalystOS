@echo off
"C:\Users\ROG G532 LV\.gemini\antigravity\scratch\qemu\qemu-system-x86_64.exe" -drive format=raw,file="target\x86_64-catalyst\debug\catalyst-kernel.img" -m 512M -display none -serial stdio -device virtio-net-pci,netdev=net0 -netdev user,id=net0 -device intel-hda,id=sound0 -device hda-duplex,bus=sound0.0 -rtc base=localtime > qemu_output.txt 2>&1
