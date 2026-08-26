use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use core::arch::naked_asm;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(crate::arch::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.divide_error.set_handler_fn(divide_error_handler);
        
        // IRQ 0 = Timer
        idt[crate::arch::interrupts::InterruptIndex::Timer.as_u8()].set_handler_fn(crate::arch::interrupts::timer_interrupt_handler);
        // IRQ 1 = Keyboard
        idt[crate::arch::interrupts::InterruptIndex::Keyboard.as_u8()].set_handler_fn(crate::arch::interrupts::keyboard_interrupt_handler);
        // IRQ 12 = Mouse
        idt[crate::arch::interrupts::InterruptIndex::Mouse.as_u8()].set_handler_fn(crate::arch::interrupts::mouse_interrupt_handler);
        
        // SYSCALL (int 0x80)
        unsafe { idt[128].set_handler_fn(syscall_handler).set_privilege_level(x86_64::PrivilegeLevel::Ring3); }
        
        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::kprintln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

fn is_user_fault(stack_frame: &InterruptStackFrame) -> bool {
    // x86_64 crate InterruptStackFrame exposes code_segment (SegmentSelector)
    (stack_frame.code_segment.0 & 3) == 3
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    if is_user_fault(&stack_frame) {
        crate::kprintln!("[USER FAULT] Page Fault at instruction: {:#x}, Code: {:?}", stack_frame.instruction_pointer.as_u64(), error_code);
        crate::task::scheduler::terminate_current_thread();
    } else {
        unsafe { crate::console::SERIAL.force_unlock(); }
        crate::kprintln!("\n[KERNEL FAULT] EXCEPTION: PAGE FAULT\n{:#?}\nError Code: {:?}", stack_frame, error_code);
        for _ in 0..100000000 { unsafe { core::arch::asm!("nop"); } } crate::console::shutdown(); loop {}
    }
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    unsafe { crate::console::SERIAL.force_unlock(); }
    crate::kprintln!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    for _ in 0..100000000 { unsafe { core::arch::asm!("nop"); } } crate::console::shutdown(); loop {}
}

extern "x86-interrupt" fn gpf_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    if is_user_fault(&stack_frame) {
        crate::kprintln!("[USER FAULT] GPF at instruction: {:#x}, Code: {:?}", stack_frame.instruction_pointer.as_u64(), error_code);
        crate::task::scheduler::terminate_current_thread();
    } else {
        unsafe { crate::console::SERIAL.force_unlock(); }
        crate::kprintln!("\n[KERNEL FAULT] EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError Code: {:?}", stack_frame, error_code);
        for _ in 0..100000000 { unsafe { core::arch::asm!("nop"); } } crate::console::shutdown(); loop {}
    }
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    if is_user_fault(&stack_frame) {
        crate::kprintln!("[USER FAULT] Invalid Opcode (#UD) at instruction: {:#x}", stack_frame.instruction_pointer.as_u64());
        crate::task::scheduler::terminate_current_thread();
    } else {
        unsafe { crate::console::SERIAL.force_unlock(); }
        crate::kprintln!("\n[KERNEL FAULT] EXCEPTION: INVALID OPCODE (#UD)\n{:#?}", stack_frame);
        for _ in 0..100000000 { unsafe { core::arch::asm!("nop"); } } crate::console::shutdown(); loop {}
    }
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    if is_user_fault(&stack_frame) {
        crate::kprintln!("[USER FAULT] Divide Error (#DE) at instruction: {:#x}", stack_frame.instruction_pointer.as_u64());
        crate::task::scheduler::terminate_current_thread();
    } else {
        unsafe { crate::console::SERIAL.force_unlock(); }
        crate::kprintln!("\n[KERNEL FAULT] EXCEPTION: DIVIDE ERROR (#DE)\n{:#?}", stack_frame);
        for _ in 0..100000000 { unsafe { core::arch::asm!("nop"); } } crate::console::shutdown(); loop {}
    }
}

extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
    crate::kprintln!("SYSCALL INT 80\n{:#?}", stack_frame);
}
