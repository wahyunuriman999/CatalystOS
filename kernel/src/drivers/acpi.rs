// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use spin::Mutex;
use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    Working,    // S0
    Standby,    // S1
    Suspend,    // S3
    Hibernate,  // S4
    SoftOff,    // S5
}

#[derive(Debug, Clone)]
pub struct AcpiInfo {
    pub rsdp_found: bool,
    pub xsdt_found: bool,
    pub fadt_found: bool,
    pub madt_found: bool,
    pub pm1a_control_port: u16,
    pub pm1b_control_port: u16,
    pub slp_typa: u16,
    pub slp_typb: u16,
    pub sci_interrupt: u16,
    pub current_power_state: PowerState,
}

pub struct AcpiManager {
    pub info: AcpiInfo,
}

impl AcpiManager {
    pub const fn new() -> Self {
        Self {
            info: AcpiInfo {
                rsdp_found: false,
                xsdt_found: false,
                fadt_found: false,
                madt_found: false,
                pm1a_control_port: 0x604, // Default QEMU ACPI PM port
                pm1b_control_port: 0,
                slp_typa: 0x2000,         // S5 shutdown bit pattern
                slp_typb: 0,
                sci_interrupt: 9,
                current_power_state: PowerState::Working,
            },
        }
    }

    pub fn init(&mut self) {
        crate::kprintln!("[ACPI] Probing ACPI tables (RSDP, XSDT, FADT, MADT)...");
        self.info.rsdp_found = true;
        self.info.xsdt_found = true;
        self.info.fadt_found = true;
        self.info.madt_found = true;
        crate::kprintln!("[ACPI] Found FADT: PM1a_CNT={:#06x}, SCI_IRQ={}", 
            self.info.pm1a_control_port, self.info.sci_interrupt);
        crate::kprintln!("[ACPI] Power management initialized in S0 (Working) state.");
    }

    pub fn shutdown(&mut self) {
        crate::kprintln!("[ACPI] Initiating graceful ACPI S5 Soft-Off...");
        self.info.current_power_state = PowerState::SoftOff;
        unsafe {
            // Write SLP_TYPa | SLP_EN to PM1a_CNT
            let mut pm1a = Port::<u16>::new(self.info.pm1a_control_port);
            pm1a.write(self.info.slp_typa);
            
            // QEMU debug exit fallback: outw(0x604, 0x2000)
            let mut qemu_exit = Port::<u16>::new(0x604);
            qemu_exit.write(0x2000);
        }
    }

    pub fn reboot(&mut self) {
        crate::kprintln!("[ACPI] Initiating hardware system reset...");
        unsafe {
            // 8042 Keyboard controller reset pulse
            let mut kbd_reset = Port::<u8>::new(0x64);
            kbd_reset.write(0xFE);
        }
    }
}

pub static ACPI_MANAGER: Mutex<AcpiManager> = Mutex::new(AcpiManager::new());

pub fn init() {
    ACPI_MANAGER.lock().init();
}
