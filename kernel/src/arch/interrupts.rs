// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use x86_64::structures::idt::InterruptStackFrame;
use pic8259::ChainedPics;
use spin;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

static TICK_COUNT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Mouse = PIC_1_OFFSET + 12,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_pics() {
    unsafe { PICS.lock().initialize() };
    init_pit();
}

pub fn init_pit() {
    use x86_64::instructions::port::Port;
    // Programmable Interval Timer (PIT) runs at 1193182 Hz.
    // Divisor for 100Hz = 1193182 / 100 = 11931 (0x2E9B)
    let divisor: u16 = 11931;
    unsafe {
        let mut command_port: Port<u8> = Port::new(0x43);
        let mut data_port: Port<u8> = Port::new(0x40);
        // Command byte: Channel 0, lobyte/hibyte, Mode 3 (Square Wave Generator), Binary mode
        command_port.write(0x36);
        data_port.write((divisor & 0xFF) as u8);
        data_port.write((divisor >> 8) as u8);
    }
}

pub fn tick_count() -> u64 {
    TICK_COUNT.load(core::sync::atomic::Ordering::Relaxed)
}

pub extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    let tick = TICK_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed) + 1;
    
    // We must send EOI before context switching, so the PIC knows we are done.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }

    // Since we re-programmed PIT to 100Hz, a tick is 10ms.
    // Let's preempt every tick (10ms timeslice).
    crate::task::do_schedule();
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::events::push_event(crate::events::RawHardwareEvent::KeyboardScancode(scancode));

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

pub extern "x86-interrupt" fn mouse_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    crate::arch::mouse::MOUSE.lock().handle_interrupt();
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
    }
}

