cd "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os"
cargo build -p catalyst-kernel
if ($LASTEXITCODE -eq 0) {
    cd "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-mkimage"
    cargo run
    cd "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os"
    $qemu = "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\qemu\qemu-system-x86_64.exe"
    $img = "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os\target\x86_64-catalyst\debug\catalyst-kernel.img"
    & $qemu -drive format=raw,file=$img -m 256M -serial stdio -display none -no-reboot -device isa-debug-exit,iobase=0xf4,iosize=0x04
}
