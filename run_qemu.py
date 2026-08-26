import subprocess
import time

print("Starting QEMU...")
p = subprocess.Popen(
    [
        r"C:\Users\ROG G532 LV\.gemini\antigravity\scratch\qemu\qemu-system-x86_64.exe",
        "-drive", r"format=raw,file=C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os\target\x86_64-catalyst\debug\catalyst-kernel.img",
        "-m", "512M",
        "-serial", "file:kernel_serial.log",
        "-no-reboot",
        "-display", "none",
        "-monitor", "stdio"
    ],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True
)

time.sleep(5)
p.stdin.write("quit\n")
p.stdin.flush()
p.wait(timeout=5)
