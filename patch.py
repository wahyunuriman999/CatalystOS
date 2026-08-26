with open('kernel/src/main.rs', 'r') as f:
    content = f.read()

content = content.replace(
    'crate::kprintln!(\"[COMPAT] Jumping to Win32 entry point...\");',
    'crate::kprintln!(\"[COMPAT] Jumping to Win32 entry point...\"); unsafe { *(pe.entry_point as *mut u16) = 0xFEEB; }'
)

with open('kernel/src/main.rs', 'w') as f:
    f.write(content)
