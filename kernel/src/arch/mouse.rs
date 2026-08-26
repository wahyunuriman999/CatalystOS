// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use spin::Mutex;
use x86_64::instructions::port::Port;
use core::sync::atomic::Ordering;

// Core mouse state is self-contained.

pub struct Ps2Mouse {
    data_port: Port<u8>,
    cmd_port: Port<u8>,
    cycle: u8,
    packet: [u8; 3],
}

impl Ps2Mouse {
    pub const fn new() -> Self {
        Self {
            data_port: Port::new(0x60),
            cmd_port: Port::new(0x64),
            cycle: 0,
            packet: [0; 3],
        }
    }

    fn wait_write(&mut self) {
        for _ in 0..10000 {
            if (unsafe { self.cmd_port.read() } & 2) == 0 {
                return;
            }
        }
    }

    fn wait_read(&mut self) {
        for _ in 0..10000 {
            if (unsafe { self.cmd_port.read() } & 1) == 1 {
                return;
            }
        }
    }

    fn write_reg(&mut self, val: u8) {
        self.wait_write();
        unsafe { self.cmd_port.write(0xD4) };
        self.wait_write();
        unsafe { self.data_port.write(val) };
    }

    fn read_reg(&mut self) -> u8 {
        self.wait_read();
        unsafe { self.data_port.read() }
    }

    pub fn init(&mut self) {
        // Enable auxiliary mouse device
        self.wait_write();
        unsafe { self.cmd_port.write(0xA8) };
        
        // Get Compac status byte
        self.wait_write();
        unsafe { self.cmd_port.write(0x20) };
        let mut status = self.read_reg();
        
        // Enable IRQ12
        status |= 2;
        status &= !0x20;
        
        self.wait_write();
        unsafe { self.cmd_port.write(0x60) };
        self.wait_write();
        unsafe { self.data_port.write(status) };
        
        // Set default settings
        self.write_reg(0xF6);
        self.read_reg(); // Ack
        
        // Enable packet streaming
        self.write_reg(0xF4);
        self.read_reg(); // Ack
    }

    pub fn handle_interrupt(&mut self) {
        let byte = unsafe { self.data_port.read() };
        crate::events::push_event(crate::events::RawHardwareEvent::MouseByte(byte));
    }
}

pub static MOUSE: Mutex<Ps2Mouse> = Mutex::new(Ps2Mouse::new());

pub fn init() {
    MOUSE.lock().init();
}
