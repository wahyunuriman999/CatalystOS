import subprocess
import time

print("Starting QEMU...")
p = subprocess.Popen(
    [
        r"C:\Users\ROG G532 LV\.gemini\antigravity\scratch\qemu\qemu-system-x86_64.exe",
        "-drive", r"format=raw,file=C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os\target\x86_64-catalyst\debug\catalyst-kernel.img",
        "-m", "512M",
        "-serial", "stdio",
        "-no-reboot",
        "-display", "none",
    ],
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True
)

print("Waiting for boot...")
time.sleep(3)

print("Killing QEMU...")
p.kill()
p.wait()

out, err = p.communicate()
print("STDOUT:")
print(out)
print("STDERR:")
print(err)
