// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

pub mod pci;
pub mod virtio_net;
pub mod virtio_block;
pub mod hda_audio;

pub fn init() {
    crate::kprintln!("---------- M8: Hardware Drivers ----------");
    pci::enumerate();
    virtio_net::init();
    virtio_block::init();
    hda_audio::init();
    crate::kprintln!("[DRIVERS] All hardware drivers initialized.");
}
