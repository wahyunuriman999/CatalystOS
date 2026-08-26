// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

pub mod color;
pub mod canvas;
pub mod geometry;
pub mod windowing;
pub mod desktop;

pub fn init_gpu() {
    crate::kprintln!("[GPU] Graphics subsystem initializing...");
    // Force initialize the desktop root windows
    desktop::DESKTOP.lock().init();
    crate::kprintln!("[GPU] Graphics subsystem initialized");
}
