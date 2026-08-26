// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

pub mod win32;
pub mod pe_loader;
pub mod registry;
pub mod posix;

static mut WIN32_ENTRY: u64 = 0;

pub fn init() {
    crate::kprintln!("---------- M9: CatalystWin Compatibility Layer ----------");
    win32::init();
    registry::init_default_keys();
    posix::init();
    crate::kprintln!("[COMPAT] CatalystWin(TM) v0.1 ready.");
    crate::kprintln!("[COMPAT] Windows PE/EXE loading: ARMED");
}
