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
pub mod acpi;
pub mod device_model;

pub fn init() {
    crate::kprintln!("---------- Hardware Drivers & Discovery ----------");
    acpi::init();
    pci::enumerate();
    device_model::init_device_tree();
    virtio_net::init();
    virtio_block::init();
    hda_audio::init();
    crate::kprintln!("[DRIVERS] All hardware drivers and discovery trees active.");
}
