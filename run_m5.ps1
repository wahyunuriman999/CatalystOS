# ==========================================
# AEGIS COGNITIVE RUNTIME PLATFORM
# PROPRIETARY AND CONFIDENTIAL
# Copyright (c) 2024-2026 Wahyu Nur Iman. 
# All rights reserved.
# ==========================================

cd "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os"

$idt_content = @'
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::arch::gdt;
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
    crate::kprintln!("[ARCH] IDT initialized.");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::kprintln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    crate::kprintln!("EXCEPTION: PAGE FAULT\n{:#?}\nError Code: {:?}", stack_frame, error_code);
    crate::console::shutdown();
    loop {}
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    crate::kprintln!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    crate::console::shutdown();
    panic!("EXCEPTION: DOUBLE FAULT");
}

extern "x86-interrupt" fn gpf_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    crate::kprintln!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError Code: {}", stack_frame, error_code);
    crate::console::shutdown();
    loop {}
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    crate::kprintln!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    crate::console::shutdown();
    loop {}
}
'@
Set-Content -Path "kernel\src\arch\idt.rs" -Value $idt_content

$syscall_content = @'
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, Star, SFMask};
use x86_64::instructions::segmentation::Segment;
use x86_64::structures::gdt::SegmentSelector;
use core::arch::naked_asm;

pub fn init(kernel_code: SegmentSelector, kernel_data: SegmentSelector, user_code: SegmentSelector, user_data: SegmentSelector) {
    unsafe {
        // Enable syscall instruction
        Efer::update(|flags| flags.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
        
        // Setup segment selectors for syscall/sysret
        // Star expects: (user_code_32_bit, kernel_code_64_bit)
        // Wait, Star layout: bits 32-47 = kernel code, bits 48-63 = user code (for sysret)
        Star::write(user_code, user_data, kernel_code, kernel_data).unwrap();
        
        // Set syscall entry point
        LStar::write(x86_64::VirtAddr::new(syscall_entry as u64));
        
        // Mask interrupts when entering syscall
        SFMask::write(x86_64::registers::rflags::RFlags::INTERRUPT_FLAG);
    }
    crate::kprintln!("[SYSCALL] System calls initialized.");
}

#[unsafe(naked)]
extern "C" fn syscall_entry() {
    unsafe {
        naked_asm!(
            "push rcx", // rcx contains RIP
            "push r11", // r11 contains RFLAGS
            
            // For a real OS we'd swapgs and set kernel stack here.
            // For now, just call handler
            
            "call {}", // call syscall_handler
            
            "pop r11",
            "pop rcx",
            "sysretq",
            sym syscall_handler
        );
    }
}

extern "C" fn syscall_handler(sys_no: u64, arg1: u64, arg2: u64) -> u64 {
    crate::kprintln!("Syscall received! No: {}, arg1: {}, arg2: {}", sys_no, arg1, arg2);
    if sys_no == 1 {
        crate::kprintln!("User mode says hello via Syscall!");
        crate::console::shutdown(); // Successfully completed userland -> kernel transition!
    }
    0
}
'@
Set-Content -Path "kernel\src\arch\syscall.rs" -Value $syscall_content

$process_content = @'
use core::arch::asm;
use x86_64::structures::gdt::SegmentSelector;

pub fn enter_usermode(entry_point: u64, user_code: SegmentSelector, user_data: SegmentSelector, stack_pointer: u64) -> ! {
    crate::kprintln!("[SCHED] Jumping to Userland (Ring 3) at {:#x}", entry_point);
    unsafe {
        asm!(
            "push {udata}",
            "push {stack}",
            "push 0x202", // RFLAGS with interrupts enabled
            "push {ucode}",
            "push {entry}",
            "iretq",
            udata = in(reg) user_data.0 as u64,
            stack = in(reg) stack_pointer,
            ucode = in(reg) user_code.0 as u64,
            entry = in(reg) entry_point,
            options(noreturn)
        );
    }
}
'@
Set-Content -Path "kernel\src\task\process.rs" -Value $process_content

$gdt_content = Get-Content -Path "kernel\src\arch\gdt.rs" -Raw
if ( -notmatch "pub user_code_selector") {
     =  -replace "let kernel_data_selector = gdt.append\(Descriptor::kernel_data_segment\(\)\);", "let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());
        let user_code_selector = gdt.append(Descriptor::user_code_segment());
        let user_data_selector = gdt.append(Descriptor::user_data_segment());"
     =  -replace "pub kernel_data_selector: SegmentSelector,", "pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,"
     =  -replace "kernel_data_selector,
            tss_selector", "kernel_data_selector,
            user_code_selector,
            user_data_selector,
            tss_selector"
    Set-Content -Path "kernel\src\arch\gdt.rs" -Value 
}

$main_content = @'
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

mod console;
mod memory;
mod arch;
mod task;

use bootloader_api::{entry_point, BootInfo};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    console::init(boot_info);
    
    crate::kprintln!("+--------------------------------------+");
    crate::kprintln!("¦        Catalyst OS v0.0.4            ¦");
    crate::kprintln!("¦  Efficient by default.               ¦");
    crate::kprintln!("¦  Responsive by design.               ¦");
    crate::kprintln!("¦  Power when you need it.             ¦");
    crate::kprintln!("+--------------------------------------+");
    crate::kprintln!("");
    
    memory::init(boot_info);
    arch::init();
    
    crate::kprintln!("---------- M5: Syscall & Userland ----------");
    
    // Test transitioning to user mode
    let user_stack = [0u8; 4096];
    let user_stack_ptr = user_stack.as_ptr() as u64 + 4096;
    
    // Get selectors from GDT
    let selectors = arch::gdt::get_selectors();
    
    // Initialize Syscall MSRs
    arch::syscall::init(
        selectors.kernel_code_selector,
        selectors.kernel_data_selector,
        selectors.user_code_selector,
        selectors.user_data_selector
    );
    
    task::process::enter_usermode(
        user_mode_func as u64,
        selectors.user_code_selector,
        selectors.user_data_selector,
        user_stack_ptr
    );
}

extern "C" fn user_mode_func() {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 1, // sys_no = 1
            in("rdi") 42,
            in("rsi") 24,
            options(nostack, preserves_flags)
        );
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    crate::kprintln!("KERNEL PANIC: {:#?}", _info);
    crate::console::shutdown();
    loop {}
}
'@
Set-Content -Path "kernel\src\main.rs" -Value $main_content

$gdt_add_get = @'
pub fn get_selectors() -> Selectors {
    GDT.1.clone()
}
'@
Add-Content -Path "kernel\src\arch\gdt.rs" -Value $gdt_add_get

$mod_arch_content = @'
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod syscall;

pub fn init() {
    crate::kprintln!("---------- M3: Interrupts & Timer ----------");
    crate::kprintln!("[ARCH] Initializing architecture...");
    gdt::init();
    idt::init();
    interrupts::init();
    crate::kprintln!("[ARCH] Architecture initialization complete.");
}
'@
Set-Content -Path "kernel\src\arch\mod.rs" -Value $mod_arch_content

cargo build -p catalyst-kernel
if ( -eq 0) {
    cd "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-mkimage"
    cargo run
    cd "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os"
     = "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\qemu\qemu-system-x86_64.exe"
     = "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os\target\x86_64-catalyst\debug\catalyst-kernel.img"
    &  -drive "format=raw,file=" -m 256M -serial stdio -display none -no-reboot -device isa-debug-exit,iobase=0xf4,iosize=0x04 > boot_output.txt 2>&1
    Get-Content boot_output.txt -Tail 50
}
