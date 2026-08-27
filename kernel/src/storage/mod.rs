// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

pub mod vfs;
pub mod ramfs;
pub mod block;
pub mod package;
pub mod update;
pub mod cpfs;

use alloc::sync::Arc;
pub use vfs::{VFS, VfsError, VNodeType, vfs_open, vfs_mkdir, vfs_unlink, O_RDONLY, O_WRONLY, O_RDWR, O_CREAT, O_TRUNC};
pub use ramfs::RamFS;
pub use block::{BlockDevice, RamDisk};
pub use package::{PackageHeader, PackageError, install_package};
pub use update::{UpdateDescriptor, SystemSlot, UpdateStatus};
pub use cpfs::{CpfsFileSystem, CpfsSuperblock, CPFS_MAGIC};

pub fn init() {
    crate::kprintln!("[STORAGE] Initializing Virtual File System (VFS)...");
    
    // Create and mount root RamFS
    let root_fs = Arc::new(RamFS::new());
    VFS.lock().mount_root(root_fs);
    
    // Create standard system directories
    let dirs = ["/bin", "/dev", "/etc", "/home", "/tmp", "/var", "/sys", "/proc"];
    for dir in dirs.iter() {
        let _ = vfs_mkdir(dir);
    }
    
    crate::kprintln!("[STORAGE] VFS Root & System directories mounted successfully.");
}
