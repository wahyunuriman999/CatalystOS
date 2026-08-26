// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityError {
    QuotaExceeded(&'static str),
    PermissionDenied(&'static str),
    NonCanonicalAddress,
    WxViolation,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessQuota {
    pub max_endpoints: usize,
    pub max_open_files: usize,
    pub max_memory_pages: usize,
    pub cur_endpoints: usize,
    pub cur_open_files: usize,
    pub cur_memory_pages: usize,
}

impl ProcessQuota {
    pub const DEFAULT: ProcessQuota = ProcessQuota {
        max_endpoints: 64,
        max_open_files: 256,
        max_memory_pages: 4096, // 16 MiB max user memory default
        cur_endpoints: 0,
        cur_open_files: 0,
        cur_memory_pages: 0,
    };

    pub fn check_allocate_endpoint(&mut self) -> Result<(), SecurityError> {
        if self.cur_endpoints >= self.max_endpoints {
            Err(SecurityError::QuotaExceeded("Endpoint quota exceeded"))
        } else {
            self.cur_endpoints += 1;
            Ok(())
        }
    }

    pub fn check_allocate_page(&mut self) -> Result<(), SecurityError> {
        if self.cur_memory_pages >= self.max_memory_pages {
            Err(SecurityError::QuotaExceeded("Memory page quota exceeded"))
        } else {
            self.cur_memory_pages += 1;
            Ok(())
        }
    }

    pub fn release_endpoint(&mut self) {
        self.cur_endpoints = self.cur_endpoints.saturating_sub(1);
    }

    pub fn release_page(&mut self) {
        self.cur_memory_pages = self.cur_memory_pages.saturating_sub(1);
    }
}

/// Validates W^X invariant (no page can be simultaneously Writable and Executable).
pub fn validate_wx_flags(is_writable: bool, is_executable: bool) -> Result<(), SecurityError> {
    if is_writable && is_executable {
        Err(SecurityError::WxViolation)
    } else {
        Ok(())
    }
}

/// Validates that an address is canonical on x86_64.
pub fn validate_canonical_address(addr: u64) -> Result<(), SecurityError> {
    let top_bits = addr >> 47;
    if top_bits != 0 && top_bits != 0x1FFFF {
        Err(SecurityError::NonCanonicalAddress)
    } else {
        Ok(())
    }
}
