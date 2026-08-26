// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

pub mod elf;
pub mod context;
pub mod process;
pub mod scheduler;
pub mod tests;

pub use scheduler::{init, spawn, do_schedule, SCHEDULE_NEEDED};
