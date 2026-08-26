pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod syscall;
pub mod mouse;

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
