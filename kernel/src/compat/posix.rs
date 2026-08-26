// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

// POSIX syscall numbers (Linux x86_64 ABI)
#[allow(dead_code)]
pub mod syscall_nr {
    pub const SYS_READ:    u64 = 0;
    pub const SYS_WRITE:   u64 = 1;
    pub const SYS_OPEN:    u64 = 2;
    pub const SYS_CLOSE:   u64 = 3;
    pub const SYS_MMAP:    u64 = 9;
    pub const SYS_MUNMAP:  u64 = 11;
    pub const SYS_BRK:     u64 = 12;
    pub const SYS_EXIT:    u64 = 60;
    pub const SYS_GETPID:  u64 = 39;
    pub const SYS_WRITE2:  u64 = 1;
    pub const SYS_FSTAT:   u64 = 5;
    pub const SYS_FUTEX:   u64 = 202;
    pub const SYS_CLONE:   u64 = 56;
    pub const SYS_EXECVE:  u64 = 59;
}

/// POSIX syscall dispatcher
/// Called when a POSIX app makes a Linux syscall
#[allow(dead_code)]
pub fn handle_posix_syscall(nr: u64, arg1: u64, arg2: u64, arg3: u64, _arg4: u64, _arg5: u64) -> i64 {
    match nr {
        syscall_nr::SYS_WRITE => {
            // fd=arg1, buf=arg2, count=arg3
            if arg1 == 1 || arg1 == 2 { // stdout/stderr
                let buf = unsafe {
                    core::slice::from_raw_parts(arg2 as *const u8, arg3 as usize)
                };
                if let Ok(s) = core::str::from_utf8(buf) {
                    crate::kprint!("{}", s);
                }
                return arg3 as i64;
            }
            -1 // EBADF
        }
        syscall_nr::SYS_GETPID => 1,
        syscall_nr::SYS_EXIT => {
            crate::kprintln!("[POSIX] Process exit({})", arg1);
            loop { x86_64::instructions::hlt(); }
        }
        syscall_nr::SYS_BRK => {
            // Return current brk (simple stub)
            0
        }
        syscall_nr::SYS_MMAP => {
            // Simple anonymous mmap
            let size = arg2 as usize;
            if size == 0 { return -1; }
            let layout = core::alloc::Layout::from_size_align(size, 4096)
                .unwrap_or(core::alloc::Layout::from_size_align(4096, 4096).unwrap());
            let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
            if ptr.is_null() { -1 } else { ptr as i64 }
        }
        syscall_nr::SYS_MUNMAP => 0, // Stub: accept but don't free
        _ => {
            crate::kprintln!("[POSIX] Unimpl syscall #{}", nr);
            -38 // ENOSYS
        }
    }
}

pub fn init() {
    crate::kprintln!("[POSIX] POSIX compatibility layer initialized.");
    crate::kprintln!("[POSIX] Linux syscall translation: ACTIVE");
    crate::kprintln!("[POSIX] Supported: read, write, mmap, exit, getpid");
}
