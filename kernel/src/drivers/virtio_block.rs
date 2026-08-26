// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use super::pci;

const VIRTIO_VENDOR: u16 = 0x1AF4;
const VIRTIO_BLK_DEVICE: u16 = 0x1001;

pub struct BlockDevice {
    pub initialized: bool,
    pub capacity_sectors: u64,
    pub io_base: u16,
}

pub static BLOCK: spin::Mutex<BlockDevice> = spin::Mutex::new(BlockDevice {
    initialized: false,
    capacity_sectors: 0,
    io_base: 0,
});

pub fn init() {
    crate::kprintln!("[BLK-DRV] Initializing VirtIO Block driver...");
    
    if let Some((bus, dev, func)) = pci::find_device(VIRTIO_VENDOR, VIRTIO_BLK_DEVICE) {
        crate::kprintln!("[BLK-DRV] Found VirtIO-blk at PCI {:02x}:{:02x}.{}", bus, dev, func);
        let bar0 = pci::pci_read32(bus, dev, func, 0x10);
        let io_base = (bar0 & !0x3) as u16;
        
        let mut blk = BLOCK.lock();
        blk.io_base = io_base;
        blk.initialized = true;
        blk.capacity_sectors = 2048; // 1MB default for now
        
        crate::kprintln!("[BLK-DRV] VirtIO-blk ready ({} sectors = {} MB)",
            blk.capacity_sectors, blk.capacity_sectors / 2048);
    } else {
        // Also check for QEMU AHCI/IDE
        if let Some((bus, dev, func)) = pci::find_class(0x01, 0x06) { // SATA
            crate::kprintln!("[BLK-DRV] Found AHCI SATA controller at {:02x}:{:02x}.{}", bus, dev, func);
        } else if let Some((bus, dev, func)) = pci::find_class(0x01, 0x01) { // IDE
            crate::kprintln!("[BLK-DRV] Found IDE controller at {:02x}:{:02x}.{}", bus, dev, func);
        } else {
            crate::kprintln!("[BLK-DRV] No block device found.");
        }
    }
}
