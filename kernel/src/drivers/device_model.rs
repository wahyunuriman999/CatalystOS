// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusType {
    Pci,
    Acpi,
    SystemBus,
    VirtIo,
    Virtual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Storage,
    Display,
    Keyboard,
    Mouse,
    Network,
    Audio,
    PowerManagement,
    SystemControl,
}

#[derive(Debug, Clone)]
pub struct DeviceDescriptor {
    pub id: DeviceId,
    pub name: String,
    pub bus: BusType,
    pub class: DeviceClass,
    pub io_base: u64,
    pub irq: u8,
    pub active: bool,
}

pub struct DeviceManager {
    devices: Vec<DeviceDescriptor>,
    next_id: u32,
}

impl DeviceManager {
    pub const fn new() -> Self {
        Self {
            devices: Vec::new(),
            next_id: 1,
        }
    }

    pub fn register_device(&mut self, name: &str, bus: BusType, class: DeviceClass, io_base: u64, irq: u8) -> DeviceId {
        let id = DeviceId(self.next_id);
        self.next_id += 1;
        self.devices.push(DeviceDescriptor {
            id,
            name: String::from(name),
            bus,
            class,
            io_base,
            irq,
            active: true,
        });
        crate::kprintln!("[DEV-MGR] Registered Device #{}: '{}' (Bus: {:?}, Class: {:?}, I/O: {:#06x}, IRQ: {})",
            id.0, name, bus, class, io_base, irq);
        id
    }

    pub fn find_by_class(&self, class: DeviceClass) -> Vec<DeviceDescriptor> {
        self.devices.iter().filter(|d| d.class == class && d.active).cloned().collect()
    }

    pub fn total_devices(&self) -> usize {
        self.devices.len()
    }
}

pub static DEVICE_MANAGER: Mutex<DeviceManager> = Mutex::new(DeviceManager::new());

pub fn init_device_tree() {
    crate::kprintln!("[DEV-MGR] Initializing Unified Device Tree...");
    let mut dm = DEVICE_MANAGER.lock();
    dm.register_device("PS/2 Keyboard Controller", BusType::SystemBus, DeviceClass::Keyboard, 0x60, 1);
    dm.register_device("PS/2 Auxiliary Mouse", BusType::SystemBus, DeviceClass::Mouse, 0x60, 12);
    dm.register_device("VirtIO Block Storage Device", BusType::VirtIo, DeviceClass::Storage, 0xC000, 11);
    dm.register_device("Linear Framebuffer Display Engine", BusType::Pci, DeviceClass::Display, 0xE000_0000, 0);
    dm.register_device("ACPI Power & Sleep Controller", BusType::Acpi, DeviceClass::PowerManagement, 0x604, 9);
    dm.register_device("Intel HDA Audio Controller", BusType::Pci, DeviceClass::Audio, 0xC040, 10);
    crate::kprintln!("[DEV-MGR] Device tree established with {} root nodes.", dm.total_devices());
}
