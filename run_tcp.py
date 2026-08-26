import subprocess
import socket
import threading
import time

def read_tcp():
    s = socket.socket()
    for i in range(20):
        try:
            s.connect(('127.0.0.1', 4444))
            print('Connected to QEMU serial!')
            break
        except:
            time.sleep(0.5)
    else:
        print('Failed to connect')
        return
    s.settimeout(10.0)
    with open('serial_tcp.log', 'w') as logf:
        while True:
            try:
                chunk = s.recv(4096)
                if not chunk: break
                text = chunk.decode('utf-8', 'replace')
                print(text, end='', flush=True)
                logf.write(text)
                logf.flush()
            except socket.timeout:
                break
    s.close()

t = threading.Thread(target=read_tcp)
t.start()
print('Starting QEMU...')
p = subprocess.Popen([
    r'C:\Users\ROG G532 LV\.gemini\antigravity\scratch\qemu\qemu-system-x86_64.exe',
    '-drive', r'format=raw,file=C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os\target\x86_64-catalyst\debug\catalyst-kernel.img',
    '-m', '512M',
    '-serial', 'tcp:127.0.0.1:4444,server,wait',
    '-display', 'none',
    '-no-reboot'
])
t.join()
print('Thread joined, killing QEMU...')
p.kill()
