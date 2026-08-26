// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VNodeType {
    File,
    Directory,
    BlockDevice,
    CharDevice,
    Pipe,
}

#[derive(Debug, Clone, Copy)]
pub struct INode {
    pub ino: u64,
    pub size: usize,
    pub node_type: VNodeType,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub ino: u64,
    pub name: String,
    pub node_type: VNodeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsError {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    NotADirectory,
    IsADirectory,
    InvalidFd,
    IoError,
    EndOfFile,
    InvalidPath,
    NoSpace,
}

pub trait VNode: Send + Sync {
    fn inode(&self) -> INode;
    fn lookup(&self, name: &str) -> Result<Arc<dyn VNode>, VfsError>;
    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize, VfsError>;
    fn write(&self, offset: usize, buf: &[u8]) -> Result<usize, VfsError>;
    fn create(&self, name: &str, node_type: VNodeType) -> Result<Arc<dyn VNode>, VfsError>;
    fn unlink(&self, name: &str) -> Result<(), VfsError>;
    fn readdir(&self) -> Result<Vec<DirEntry>, VfsError>;
    fn truncate(&self, size: usize) -> Result<(), VfsError>;
}

pub trait FileSystem: Send + Sync {
    fn name(&self) -> &'static str;
    fn root(&self) -> Arc<dyn VNode>;
}

pub struct OpenFile {
    pub vnode: Arc<dyn VNode>,
    pub offset: usize,
    pub flags: u32,
}

pub const O_RDONLY: u32 = 0;
pub const O_WRONLY: u32 = 1;
pub const O_RDWR: u32   = 2;
pub const O_CREAT: u32  = 0x0100;
pub const O_TRUNC: u32  = 0x0200;
pub const O_APPEND: u32 = 0x0400;

pub struct FileDescriptorTable {
    fds: Vec<Option<OpenFile>>,
}

impl FileDescriptorTable {
    pub fn new() -> Self {
        FileDescriptorTable {
            fds: Vec::new(),
        }
    }

    pub fn insert(&mut self, file: OpenFile) -> Result<usize, VfsError> {
        for (i, slot) in self.fds.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(file);
                return Ok(i);
            }
        }
        let fd = self.fds.len();
        self.fds.push(Some(file));
        Ok(fd)
    }

    pub fn get(&mut self, fd: usize) -> Result<&mut OpenFile, VfsError> {
        self.fds.get_mut(fd)
            .and_then(|opt| opt.as_mut())
            .ok_or(VfsError::InvalidFd)
    }

    pub fn close(&mut self, fd: usize) -> Result<(), VfsError> {
        let slot = self.fds.get_mut(fd).ok_or(VfsError::InvalidFd)?;
        if slot.is_some() {
            *slot = None;
            Ok(())
        } else {
            Err(VfsError::InvalidFd)
        }
    }
}

pub struct VfsMount {
    pub path: String,
    pub fs: Arc<dyn FileSystem>,
}

pub struct VfsRoot {
    pub root_fs: Option<Arc<dyn FileSystem>>,
    pub mounts: Vec<VfsMount>,
}

impl VfsRoot {
    pub const fn new() -> Self {
        VfsRoot {
            root_fs: None,
            mounts: Vec::new(),
        }
    }

    pub fn mount_root(&mut self, fs: Arc<dyn FileSystem>) {
        self.root_fs = Some(fs);
    }

    pub fn mount(&mut self, path: &str, fs: Arc<dyn FileSystem>) {
        self.mounts.push(VfsMount {
            path: String::from(path),
            fs,
        });
    }

    pub fn lookup(&self, path: &str) -> Result<Arc<dyn VNode>, VfsError> {
        let root = self.root_fs.as_ref().ok_or(VfsError::NotFound)?.root();
        
        let path = path.trim();
        if path == "/" || path.is_empty() {
            return Ok(root);
        }

        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = root;

        for seg in segments {
            current = current.lookup(seg)?;
        }

        Ok(current)
    }
}

pub static VFS: Mutex<VfsRoot> = Mutex::new(VfsRoot::new());

pub fn vfs_open(path: &str, flags: u32) -> Result<Arc<dyn VNode>, VfsError> {
    let vfs = VFS.lock();
    match vfs.lookup(path) {
        Ok(vnode) => {
            if flags & O_TRUNC != 0 {
                vnode.truncate(0)?;
            }
            Ok(vnode)
        }
        Err(VfsError::NotFound) if flags & O_CREAT != 0 => {
            // Find parent and create
            let (parent_path, file_name) = split_path(path)?;
            let parent = vfs.lookup(parent_path)?;
            parent.create(file_name, VNodeType::File)
        }
        Err(e) => Err(e),
    }
}

pub fn vfs_mkdir(path: &str) -> Result<Arc<dyn VNode>, VfsError> {
    let vfs = VFS.lock();
    let (parent_path, dir_name) = split_path(path)?;
    let parent = vfs.lookup(parent_path)?;
    parent.create(dir_name, VNodeType::Directory)
}

pub fn vfs_unlink(path: &str) -> Result<(), VfsError> {
    let vfs = VFS.lock();
    let (parent_path, name) = split_path(path)?;
    let parent = vfs.lookup(parent_path)?;
    parent.unlink(name)
}

fn split_path(path: &str) -> Result<(&str, &str), VfsError> {
    let path = path.trim_end_matches('/');
    if path.is_empty() {
        return Err(VfsError::InvalidPath);
    }
    match path.rfind('/') {
        Some(0) => Ok(("/", &path[1..])),
        Some(idx) => Ok((&path[..idx], &path[idx + 1..])),
        None => Ok(("/", path)),
    }
}
