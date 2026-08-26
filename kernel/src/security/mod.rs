// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

pub mod quota;
pub mod watchdog;

pub use quota::{ProcessQuota, SecurityError, validate_wx_flags, validate_canonical_address};
pub use watchdog::{Watchdog, KERNEL_WATCHDOG};

pub fn init_security() {
    crate::kprintln!("[SECURITY] Hardening subsystem active: W^X, Canonical Addresses, Process Quotas.");
}
