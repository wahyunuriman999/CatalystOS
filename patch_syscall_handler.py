import re

with open('kernel/src/arch/syscall.rs', 'r') as f:
    content = f.read()

new_handler = '''extern "C" fn syscall_handler(sys_no: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    crate::kprintln!("[SYSCALL] No: {}, Args: {:#x}, {:#x}, {:#x}, {:#x}", sys_no, arg1, arg2, arg3, arg4);
    
    if sys_no == 102 { // GetStdHandle
        crate::kprintln!("[WIN32] GetStdHandle called!");
        return 0xFFFFFFF5; // Return dummy handle
    }
    else if sys_no == 101 { // WriteConsoleA
        crate::kprintln!("[WIN32] WriteConsoleA called!");
        // The message is at rg2, length is rg3.
        let msg_ptr = arg2 as *const u8;
        let msg_len = arg3 as usize;
        let slice = unsafe { core::slice::from_raw_parts(msg_ptr, msg_len) };
        if let Ok(s) = core::str::from_utf8(slice) {
            crate::kprintln!("[WIN32 OUTPUT]: {}", s);
        }
        return 1; // Success
    }
    else if sys_no == 103 { // ExitProcess
        crate::kprintln!("[WIN32] ExitProcess called with code: {}", arg1);
        crate::kprintln!("*** CATALYST OS - WIN32 EXECUTION SUCCESSFUL! ***");
        crate::console::shutdown(); // Exit QEMU successfully!
    }
    
    0
}'''

content = re.sub(r'extern "C" fn syscall_handler.*?\n}', new_handler, content, flags=re.DOTALL)

with open('kernel/src/arch/syscall.rs', 'w') as f:
    f.write(content)
