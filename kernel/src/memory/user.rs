// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use x86_64::VirtAddr;

pub const USER_SPACE_LIMIT: u64 = 0x0000_7FFF_FFFF_FFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    InvalidAddress,
    KernelAddressAccessViolation,
    NotPresent,
    PermissionDenied,
}

/// Validates that a user memory range resides completely below the kernel boundary (C9, Rule 4).
pub fn validate_user_buffer(ptr: u64, len: usize) -> Result<(), MemoryError> {
    if len == 0 {
        return Ok(());
    }
    
    let end = ptr.checked_add(len as u64).ok_or(MemoryError::InvalidAddress)?;
    
    // User address space must be canonical and strictly below USER_SPACE_LIMIT
    if ptr >= USER_SPACE_LIMIT || end > USER_SPACE_LIMIT || ptr == 0 {
        return Err(MemoryError::KernelAddressAccessViolation);
    }
    
    Ok(())
}

/// Safely copy data from userspace buffer into a kernel slice (C9 Copy-In semantics).
pub fn copy_from_user(user_src: *const u8, kernel_dst: &mut [u8]) -> Result<(), MemoryError> {
    let src_addr = user_src as u64;
    validate_user_buffer(src_addr, kernel_dst.len())?;
    
    // Perform copy
    unsafe {
        core::ptr::copy_nonoverlapping(user_src, kernel_dst.as_mut_ptr(), kernel_dst.len());
    }
    
    Ok(())
}

/// Safely copy data from kernel buffer into userspace buffer (C9 Copy-Out semantics).
pub fn copy_to_user(kernel_src: &[u8], user_dst: *mut u8) -> Result<(), MemoryError> {
    let dst_addr = user_dst as u64;
    validate_user_buffer(dst_addr, kernel_src.len())?;
    
    // Perform copy
    unsafe {
        core::ptr::copy_nonoverlapping(kernel_src.as_ptr(), user_dst, kernel_src.len());
    }
    
    Ok(())
}
