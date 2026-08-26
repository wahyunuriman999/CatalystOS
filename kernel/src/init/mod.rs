// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

pub mod services;

pub use services::{SERVICE_MANAGER, ServiceManager, ServiceState, init_services};

pub fn start_init_process() {
    crate::kprintln!("[INIT] Initializing system services subsystem...");
    init_services();
}
