use x86_64::registers::model_specific::{Efer, EferFlags, LStar, Star, SFMask, KernelGsBase};
use x86_64::instructions::segmentation::Segment;
use x86_64::structures::gdt::SegmentSelector;
use core::arch::naked_asm;

#[repr(C)]
pub struct CpuLocal {
    pub kernel_rsp: u64,
    pub user_rsp: u64,
}

#[unsafe(no_mangle)]
pub static mut CPU_LOCAL: CpuLocal = CpuLocal { kernel_rsp: 0, user_rsp: 0 };

pub fn init(
    kernel_code: SegmentSelector,
    _kernel_data: SegmentSelector,
    user_code_32: SegmentSelector,
    _user_data: SegmentSelector
) {
    unsafe {
        let priv_stack_top = 0x200000;
        CPU_LOCAL.kernel_rsp = priv_stack_top; 
        KernelGsBase::write(x86_64::VirtAddr::new(&raw const CPU_LOCAL as u64));

        Efer::update(|flags| flags.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
        
        let mut star_msr = x86_64::registers::model_specific::Msr::new(0xC000_0081);
        let mut star_val = 0u64;
        star_val |= (kernel_code.0 as u64) << 32;
        star_val |= ((user_code_32.0 as u64) | 3) << 48;
        star_msr.write(star_val);
        
        LStar::write(x86_64::VirtAddr::new(syscall_entry as *const () as u64));
        
        SFMask::write(x86_64::registers::rflags::RFlags::INTERRUPT_FLAG);
    }
    crate::kprintln!("[SYSCALL] System calls initialized.");
}

#[unsafe(naked)]
extern "C" fn syscall_entry() {
    unsafe {
        naked_asm!(
            "swapgs",
            "mov gs:[8], rsp",
            "mov rsp, gs:[0]",

            "push rcx",
            "push r11",
            "push rbp",
            "mov rbp, rsp",
              
            "mov rdi, rax",
            "mov rsi, r10",
            "mov rcx, r8",
            "mov r8, r9",
              
            "call {}",
              
            "mov rsp, rbp",
            "pop rbp",
            "pop r11",
            "pop rcx",
            
            "mov rsp, gs:[8]",
            "swapgs",
            "sysretq",
            sym syscall_handler
        );
    }
}

extern "C" fn syscall_handler(sys_no: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    crate::kprintln!("[SYSCALL] No: {}, Args: {:#x}, {:#x}, {:#x}, {:#x}", sys_no, arg1, arg2, arg3, arg4);
    
    if sys_no == 102 { // GetStdHandle
        crate::kprintln!("[WIN32] GetStdHandle called!");
        return 0xFFFFFFF5;
    }
    else if sys_no == 101 { // WriteConsoleA
        crate::kprintln!("[WIN32] WriteConsoleA called!");
        let msg_ptr = arg2 as *const u8;
        let msg_len = arg3 as usize;
        let slice = unsafe { core::slice::from_raw_parts(msg_ptr, msg_len) };
        if let Ok(s) = core::str::from_utf8(slice) {
            crate::kprintln!("[WIN32 OUTPUT]: {}", s);
        }
        return 1;
    }
    else if sys_no == 103 { // ExitProcess
        crate::kprintln!("[WIN32] ExitProcess called with code: {}", arg1);
        crate::kprintln!("*** CATALYST OS - WIN32 EXECUTION SUCCESSFUL! ***");
        crate::console::shutdown();
    }
    
    if sys_no == 1 {
        crate::kprintln!("*** SYSCALL 1 TRIGGERED FROM RING 3 ***");
        crate::console::shutdown();
    }
    0
}
