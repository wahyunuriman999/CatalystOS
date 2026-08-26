Write-Host "Building kernel..."
cargo build -p catalyst-kernel
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Building image..."
Rename-Item -Path .cargo/config.toml -NewName config_bak.toml
cargo run --manifest-path tools/mkimage/Cargo.toml
$err = $LASTEXITCODE
Rename-Item -Path .cargo/config_bak.toml -NewName config.toml
if ($err -ne 0) { exit $err }

Write-Host "Running QEMU..."
$qemu = "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\qemu\qemu-system-x86_64.exe"
$img = "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os\target\x86_64-catalyst\debug\catalyst-kernel.img"
& $qemu -drive format=raw,file=$img -m 256M -debugcon file:qemu_debug.log -global isa-debugcon.iobase=0xe9 -serial file:qemu.log -d int -D qemu_int.log -monitor tcp:127.0.0.1:5555,server,nowait -vnc 127.0.0.1:0 -no-reboot -device isa-debug-exit,iobase=0xf4,iosize=0x04 -vga std
