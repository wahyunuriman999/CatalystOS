// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use super::pci;

const INTEL_HDA_VENDOR: u16 = 0x8086;
const INTEL_HDA_DEVICE: u16 = 0x2668; // ICH6 HDA (used by QEMU)
const INTEL_HDA_DEVICE2: u16 = 0x293E; // ICH9

pub struct AudioDevice {
    pub initialized: bool,
    pub sample_rate: u32,
    pub channels: u8,
}

pub static AUDIO: spin::Mutex<AudioDevice> = spin::Mutex::new(AudioDevice {
    initialized: false,
    sample_rate: 48000,
    channels: 2,
});

pub fn init() {
    crate::kprintln!("[AUDIO] Initializing Intel HDA audio driver...");
    
    // Check for Intel HDA
    let found = pci::find_device(INTEL_HDA_VENDOR, INTEL_HDA_DEVICE)
        .or_else(|| pci::find_device(INTEL_HDA_VENDOR, INTEL_HDA_DEVICE2))
        .or_else(|| pci::find_class(0x04, 0x03)); // Audio class
    
    if let Some((bus, dev, func)) = found {
        let vendor = pci::pci_read16(bus, dev, func, 0x00);
        let device = pci::pci_read16(bus, dev, func, 0x02);
        crate::kprintln!("[AUDIO] Found audio device at {:02x}:{:02x}.{} ({:#06x}:{:#06x})",
            bus, dev, func, vendor, device);
        
        let mut audio = AUDIO.lock();
        audio.initialized = true;
        crate::kprintln!("[AUDIO] HDA ready: {}Hz stereo", audio.sample_rate);
    } else {
        crate::kprintln!("[AUDIO] No audio device found (add -device intel-hda,id=sound0 to QEMU).");
    }
}

pub fn is_available() -> bool {
    AUDIO.lock().initialized
}
