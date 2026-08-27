// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod syscall;
pub mod mouse;

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

/// A minimal static IDT loaded BEFORE heap/serial/GDT are available.
/// Just halts on every exception so we get a visible hang instead of
/// an invisible triple fault. arch::init() will overwrite this.
static mut EARLY_IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

extern "x86-interrupt" fn early_catch_all(_frame: InterruptStackFrame) {
    unsafe {
        let mut p = x86_64::instructions::port::PortWriteOnly::<u8>::new(0xe9);
        for b in b"EARLY FAULT!\r\n" { p.write(*b); }
        let mut s = x86_64::instructions::port::PortWriteOnly::<u8>::new(0x3F8);
        for b in b"EARLY FAULT!\r\n" { s.write(*b); }
    }
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn early_page_fault(
    _frame: InterruptStackFrame,
    _error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
    unsafe {
        let mut p = x86_64::instructions::port::PortWriteOnly::<u8>::new(0xe9);
        for b in b"EARLY PAGE FAULT!\r\n" { p.write(*b); }
        let mut s = x86_64::instructions::port::PortWriteOnly::<u8>::new(0x3F8);
        for b in b"EARLY PAGE FAULT!\r\n" { s.write(*b); }
    }
    loop { x86_64::instructions::hlt(); }
}

/// Load a minimal IDT immediately after kernel entry.
/// Interrupts MUST be disabled (cli) before calling this.
pub fn early_idt_init() {
    unsafe {
        let idt = core::ptr::addr_of_mut!(EARLY_IDT);
        (*idt).breakpoint.set_handler_fn(early_catch_all);
        (*idt).page_fault.set_handler_fn(early_page_fault);
        (*idt).load_unsafe();
    }
}

pub fn init() {
    crate::kprintln!("---------- M3: Interrupts & Timer ----------");
    crate::kprintln!("[ARCH] Initializing architecture...");
    
    unsafe {
        let mut cr0 = x86_64::registers::control::Cr0::read();
        cr0.remove(x86_64::registers::control::Cr0Flags::EMULATE_COPROCESSOR);
        cr0.insert(x86_64::registers::control::Cr0Flags::MONITOR_COPROCESSOR);
        x86_64::registers::control::Cr0::write(cr0);
        
        let mut cr4 = x86_64::registers::control::Cr4::read();
        cr4.insert(x86_64::registers::control::Cr4Flags::OSFXSR);
        cr4.insert(x86_64::registers::control::Cr4Flags::OSXMMEXCPT_ENABLE);
        x86_64::registers::control::Cr4::write(cr4);
    }

    gdt::init();
    idt::init();
    interrupts::init_pics(); // Rename from init to init_pics
    mouse::init();
    
    let selectors = gdt::get_selectors();
    syscall::init(selectors.kernel_code_selector, selectors.kernel_data_selector, selectors.user_code_32_selector, selectors.user_data_selector);
    
    // Enable interrupts safely after ALL infrastructure is ready
    x86_64::instructions::interrupts::enable();
    crate::kprintln!("[ARCH] Architecture initialization complete (Interrupts ENABLED).");
}

