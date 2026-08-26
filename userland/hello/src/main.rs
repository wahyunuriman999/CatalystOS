#![no_std]
#![no_main]
use core::arch::asm;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let msg = b"Hello from Catalyst Userland!";
    unsafe {
        // Syscall 1: write
        // arg1 = msg pointer, arg2 = length
        asm!(
            "syscall",
            in("rax") 1,
            in("rdi") msg.as_ptr() as u64,
            in("rsi") msg.len() as u64,
            options(nostack, preserves_flags)
        );
        
        // Syscall 60: exit
        asm!(
            "syscall",
            in("rax") 60,
            in("rdi") 0,
            options(noreturn)
        );
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
