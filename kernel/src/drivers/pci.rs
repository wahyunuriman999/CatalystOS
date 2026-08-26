// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use x86_64::instructions::port::{PortWriteOnly, PortReadOnly};

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub header_type: u8,
}

fn pci_config_address(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
    (1u32 << 31)
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
}

pub fn pci_read32(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
    unsafe {
        let mut addr_port = PortWriteOnly::<u32>::new(CONFIG_ADDRESS);
        let mut data_port = PortReadOnly::<u32>::new(CONFIG_DATA);
        addr_port.write(pci_config_address(bus, device, func, offset));
        data_port.read()
    }
}

pub fn pci_read16(bus: u8, device: u8, func: u8, offset: u8) -> u16 {
    let val = pci_read32(bus, device, func, offset & 0xFC);
    ((val >> ((offset & 2) * 8)) & 0xFFFF) as u16
}

pub fn pci_read8(bus: u8, device: u8, func: u8, offset: u8) -> u8 {
    let val = pci_read32(bus, device, func, offset & 0xFC);
    ((val >> ((offset & 3) * 8)) & 0xFF) as u8
}

pub fn pci_write32(bus: u8, device: u8, func: u8, offset: u8, value: u32) {
    unsafe {
        let mut addr_port = PortWriteOnly::<u32>::new(CONFIG_ADDRESS);
        let mut data_port = PortWriteOnly::<u32>::new(CONFIG_DATA);
        addr_port.write(pci_config_address(bus, device, func, offset));
        data_port.write(value);
    }
}

/// Known vendor IDs
const VENDOR_VIRTIO: u16 = 0x1AF4;
const VENDOR_INTEL:  u16 = 0x8086;
const VENDOR_AMD:    u16 = 0x1022;

/// PCI device class codes
const CLASS_NETWORK:     u8 = 0x02;
const CLASS_STORAGE:     u8 = 0x01;
const CLASS_DISPLAY:     u8 = 0x03;
const CLASS_MULTIMEDIA:  u8 = 0x04;
const CLASS_BRIDGE:      u8 = 0x06;
const CLASS_SERIAL:      u8 = 0x0C;

use alloc::vec::Vec;
use spin::Mutex;

pub static PCI_DEVICES: Mutex<Vec<PciDevice>> = Mutex::new(Vec::new());

pub fn enumerate() {
    crate::kprintln!("[PCI] Enumerating PCI bus...");
    let mut devices = PCI_DEVICES.lock();
    let mut count = 0;
    
    for bus_u16 in 0u16..256 { let bus = bus_u16 as u8;
        for dev in 0u8..32 {
            let vendor = pci_read16(bus, dev, 0, 0x00);
            if vendor == 0xFFFF { continue; } // No device
            
            let header_type = pci_read8(bus, dev, 0, 0x0E);
            let num_funcs = if header_type & 0x80 != 0 { 8 } else { 1 };
            
            for func in 0..num_funcs {
                let vendor_id = pci_read16(bus, dev, func, 0x00);
                if vendor_id == 0xFFFF { continue; }
                
                let device_id  = pci_read16(bus, dev, func, 0x02);
                let class_code = pci_read8(bus, dev, func, 0x0B);
                let subclass   = pci_read8(bus, dev, func, 0x0A);
                let prog_if    = pci_read8(bus, dev, func, 0x09);
                let hdr_type   = pci_read8(bus, dev, func, 0x0E);
                
                let class_name = match class_code {
                    CLASS_NETWORK    => "Network",
                    CLASS_STORAGE    => "Storage",
                    CLASS_DISPLAY    => "Display/GPU",
                    CLASS_MULTIMEDIA => "Audio/Multimedia",
                    CLASS_BRIDGE     => "Bridge",
                    CLASS_SERIAL     => "Serial Bus",
                    0x00             => "Unclassified",
                    _                => "Other",
                };
                
                crate::kprintln!(
                    "[PCI] {:02x}:{:02x}.{} VendorID={:#06x} DeviceID={:#06x} Class={} ({:02x}:{:02x})",
                    bus, dev, func, vendor_id, device_id, class_name, class_code, subclass
                );
                
                devices.push(PciDevice {
                    bus, device: dev, function: func,
                    vendor_id, device_id, class_code, subclass, prog_if,
                    header_type: hdr_type,
                });
                count += 1;
            }
        }
        // Stop early after bus 0 if no devices found on higher buses
        if bus == 0 && devices.is_empty() { break; }
        if bus > 0 && count == 0 { break; }
    }
    crate::kprintln!("[PCI] Found {} PCI devices.", count);
}

pub fn find_device(vendor: u16, device: u16) -> Option<(u8, u8, u8)> {
    let devices = PCI_DEVICES.lock();
    for d in devices.iter() {
        if d.vendor_id == vendor && d.device_id == device {
            return Some((d.bus, d.device, d.function));
        }
    }
    None
}

pub fn find_class(class: u8, subclass: u8) -> Option<(u8, u8, u8)> {
    let devices = PCI_DEVICES.lock();
    for d in devices.iter() {
        if d.class_code == class && d.subclass == subclass {
            return Some((d.bus, d.device, d.function));
        }
    }
    None
}


