import socket
from PIL import Image
import struct

def capture_vnc(host='127.0.0.1', port=5900, outfile='vnc_screen.png'):
    try:
        s = socket.create_connection((host, port))
    except Exception as e:
        print(f"Connection failed: {e}")
        return

    server_version = s.recv(12)
    s.sendall(b'RFB 003.008\n')
    
    num_security = s.recv(1)[0]
    sec_types = s.recv(num_security)
    s.sendall(bytes([sec_types[0]]))
    
    if sec_types[0] != 1:
        print("Security not None")
        return
        
    sec_result = s.recv(4)
    if struct.unpack(">I", sec_result)[0] != 0:
        print("Security failed")
        return
        
    s.sendall(b'\x01')
    
    w, h, pxl_format, name_len = struct.unpack(">HH16sI", s.recv(24))
    name = s.recv(name_len)
    
    print(f"Screen: {w}x{h}, Name: {name}")
    
    bits_per_pixel, depth, big_endian, true_color, r_max, g_max, b_max, r_shift, g_shift, b_shift = struct.unpack(">BBBBHHHBBB", pxl_format[:13])
    
    # Send SetEncodings to allow DesktopSize
    s.sendall(struct.pack(">B H", 2, 2) + struct.pack(">l", 0) + struct.pack(">l", -223))
    
    # Try to request 1280x720 directly
    req = struct.pack(">B B H H H H", 3, 0, 0, 0, 1280, 720)
    s.sendall(req)
    
    msg_type = s.recv(1)[0]
    if msg_type != 0:
        print(f"Expected FramebufferUpdate, got {msg_type}")
        return
        
    s.recv(1)
    num_rects = struct.unpack(">H", s.recv(2))[0]
    
    for _ in range(num_rects):
        rx, ry, rw, rh, enc = struct.unpack(">HHHHl", s.recv(12))
        if enc == -223:
            print(f"Desktop size changed to {rw}x{rh}")
            req = struct.pack(">B B H H H H", 3, 0, 0, 0, rw, rh)
            s.sendall(req)
            # Re-read next update
            msg_type = s.recv(1)[0]
            s.recv(1)
            num_rects2 = struct.unpack(">H", s.recv(2))[0]
            for _ in range(num_rects2):
                rx, ry, rw, rh, enc = struct.unpack(">HHHHl", s.recv(12))
                pixels = bytearray()
                expected_len = rw * rh * (bits_per_pixel // 8)
                while len(pixels) < expected_len:
                    pixels += s.recv(expected_len - len(pixels))
                img = Image.frombytes("RGBX", (rw, rh), bytes(pixels))
                img.convert('RGB').save(outfile)
                print(f"Saved {outfile} at {rw}x{rh}")
            s.close()
            return
            
        if enc != 0:
            continue
            
        pixels = bytearray()
        expected_len = rw * rh * (bits_per_pixel // 8)
        while len(pixels) < expected_len:
            pixels += s.recv(expected_len - len(pixels))
            
        img = Image.frombytes("RGBX", (rw, rh), bytes(pixels))
        img.convert('RGB').save(outfile)
        print(f"Saved {outfile} at {rw}x{rh}")
        
    s.close()

capture_vnc()

