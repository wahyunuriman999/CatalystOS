// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

pub mod quota;
pub mod watchdog;
pub mod permission_bubble;

pub use quota::{ProcessQuota, SecurityError, validate_wx_flags, validate_canonical_address};
pub use watchdog::{Watchdog, KERNEL_WATCHDOG};
pub use permission_bubble::{PermissionKind, PermissionDecision, PermissionBubble, BUBBLE_MANAGER};

pub fn init_security() {
    crate::kprintln!("[SECURITY] Hardening subsystem active: W^X, Canonical Addresses, Process Quotas, Permission Bubbles.");
}
