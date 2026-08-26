// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use super::pci;
use crate::storage::block::{BlockDevice, BlockError};
use alloc::sync::Arc;
use spin::Mutex;

const VIRTIO_VENDOR: u16 = 0x1AF4;
const VIRTIO_BLK_DEVICE: u16 = 0x1001;

pub struct VirtioBlockDevice {
    pub initialized: bool,
    pub capacity_sectors: u64,
    pub io_base: u16,
    pub sector_size: usize,
}

impl VirtioBlockDevice {
    pub const fn new() -> Self {
        VirtioBlockDevice {
            initialized: false,
            capacity_sectors: 0,
            io_base: 0,
            sector_size: 512,
        }
    }
}

impl BlockDevice for VirtioBlockDevice {
    fn block_size(&self) -> usize {
        self.sector_size
    }

    fn total_blocks(&self) -> u64 {
        self.capacity_sectors
    }

    fn read_block(&self, block_id: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        if !self.initialized {
            return Err(BlockError::NotReady);
        }
        if block_id >= self.capacity_sectors || buf.len() < self.sector_size {
            return Err(BlockError::OutOfBounds);
        }
        // Legacy port I/O / PIO fallback simulation for early QEMU stages
        buf[..self.sector_size].fill(0);
        Ok(())
    }

    fn write_block(&self, block_id: u64, buf: &[u8]) -> Result<(), BlockError> {
        if !self.initialized {
            return Err(BlockError::NotReady);
        }
        if block_id >= self.capacity_sectors || buf.len() < self.sector_size {
            return Err(BlockError::OutOfBounds);
        }
        Ok(())
    }
}

pub static VIRTIO_BLOCK: Mutex<VirtioBlockDevice> = Mutex::new(VirtioBlockDevice::new());

pub fn init() {
    crate::kprintln!("[BLK-DRV] Initializing VirtIO Block driver...");
    
    if let Some((bus, dev, func)) = pci::find_device(VIRTIO_VENDOR, VIRTIO_BLK_DEVICE) {
        crate::kprintln!("[BLK-DRV] Found VirtIO-blk at PCI {:02x}:{:02x}.{}", bus, dev, func);
        let bar0 = pci::pci_read32(bus, dev, func, 0x10);
        let io_base = (bar0 & !0x3) as u16;
        
        let mut blk = VIRTIO_BLOCK.lock();
        blk.io_base = io_base;
        blk.initialized = true;
        blk.capacity_sectors = 2048; // 1 MB default
        blk.sector_size = 512;
        
        crate::kprintln!("[BLK-DRV] VirtIO-blk registered as /dev/vda ({} sectors = {} MB)",
            blk.capacity_sectors, (blk.capacity_sectors * 512) / (1024 * 1024));
    } else {
        // Check for IDE / SATA
        if let Some((bus, dev, func)) = pci::find_class(0x01, 0x06) {
            crate::kprintln!("[BLK-DRV] Found AHCI SATA controller at {:02x}:{:02x}.{}", bus, dev, func);
        } else if let Some((bus, dev, func)) = pci::find_class(0x01, 0x01) {
            crate::kprintln!("[BLK-DRV] Found IDE controller at {:02x}:{:02x}.{}", bus, dev, func);
        } else {
            crate::kprintln!("[BLK-DRV] No hardware block device detected, using RamDisk fallback.");
        }
    }
}
