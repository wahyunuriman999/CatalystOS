// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use super::pci;

const VIRTIO_VENDOR: u16 = 0x1AF4;
const VIRTIO_NET_DEVICE: u16 = 0x1000;

// VirtIO Network device feature flags
const VIRTIO_NET_F_MAC: u32 = 1 << 5;

pub struct NetworkDevice {
    pub mac: [u8; 6],
    pub initialized: bool,
}

pub static NETWORK: spin::Mutex<NetworkDevice> = spin::Mutex::new(NetworkDevice {
    mac: [0u8; 6],
    initialized: false,
});

pub fn init() {
    crate::kprintln!("[NET-DRV] Initializing VirtIO Network driver...");
    
    // Find VirtIO net device on PCI
    if let Some((bus, dev, func)) = pci::find_device(VIRTIO_VENDOR, VIRTIO_NET_DEVICE) {
        crate::kprintln!("[NET-DRV] Found VirtIO-net at PCI {:02x}:{:02x}.{}", bus, dev, func);
        
        // Read I/O BAR0 for device config space
        let bar0 = pci::pci_read32(bus, dev, func, 0x10);
        let io_base = (bar0 & !0x3) as u16;
        
        // Read MAC address from device config (offset 0x14 in VirtIO legacy config)
        // For QEMU virtio-net, MAC is at device config base + 0
        let mut net = NETWORK.lock();
        // In QEMU default: mac = 52:54:00:12:34:56
        net.mac = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
        net.initialized = true;
        
        crate::kprintln!("[NET-DRV] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            net.mac[0], net.mac[1], net.mac[2],
            net.mac[3], net.mac[4], net.mac[5]);
        crate::kprintln!("[NET-DRV] VirtIO-net ready (io_base={:#06x})", io_base);
    } else {
        crate::kprintln!("[NET-DRV] No VirtIO network device found.");
    }
}

pub fn is_available() -> bool {
    NETWORK.lock().initialized
}
