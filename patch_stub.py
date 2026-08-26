import re

with open('kernel/src/compat/pe_loader.rs', 'r') as f:
    content = f.read()

new_stub = '''// mov r10, rcx (49 89 CA)
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET) as *mut u8) = 0x49; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 1) as *mut u8) = 0x89; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 2) as *mut u8) = 0xCA; }
        
        // mov eax, syscall_id (B8 id id id id)
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 3) as *mut u8) = 0xB8; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 4) as *mut u8) = (syscall_id & 0xFF) as u8; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 5) as *mut u8) = ((syscall_id >> 8) & 0xFF) as u8; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 6) as *mut u8) = ((syscall_id >> 16) & 0xFF) as u8; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 7) as *mut u8) = ((syscall_id >> 24) & 0xFF) as u8; }
        
        // syscall (0F 05)
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 8) as *mut u8) = 0x0F; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 9) as *mut u8) = 0x05; }
        
        // ret (C3)
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 10) as *mut u8) = 0xC3; }
        
        STUB_OFFSET += 16;'''

content = re.sub(r'// mov eax, syscall_id \(B8 id id id id\).*?STUB_OFFSET \+= 8;', new_stub, content, flags=re.DOTALL)

with open('kernel/src/compat/pe_loader.rs', 'w') as f:
    f.write(content)
