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
use core::sync::atomic::{AtomicU64, Ordering};
use crate::storage::vfs::{FileSystem, VNode, VNodeType, INode, DirEntry, VfsError};

static NEXT_INO: AtomicU64 = AtomicU64::new(1);

pub struct RamNode {
    pub ino: u64,
    pub node_type: VNodeType,
    pub data: Mutex<Vec<u8>>,
    pub children: Mutex<Vec<(String, Arc<RamNode>)>>,
}

impl RamNode {
    pub fn new(node_type: VNodeType) -> Self {
        RamNode {
            ino: NEXT_INO.fetch_add(1, Ordering::Relaxed),
            node_type,
            data: Mutex::new(Vec::new()),
            children: Mutex::new(Vec::new()),
        }
    }
}

impl VNode for RamNode {
    fn inode(&self) -> INode {
        INode {
            ino: self.ino,
            size: self.data.lock().len(),
            node_type: self.node_type,
            mode: 0o755,
            uid: 0,
            gid: 0,
        }
    }

    fn lookup(&self, name: &str) -> Result<Arc<dyn VNode>, VfsError> {
        if self.node_type != VNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        let children = self.children.lock();
        for (child_name, child_node) in children.iter() {
            if child_name == name {
                return Ok(child_node.clone() as Arc<dyn VNode>);
            }
        }
        Err(VfsError::NotFound)
    }

    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize, VfsError> {
        if self.node_type != VNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        let data = self.data.lock();
        if offset >= data.len() {
            return Ok(0); // EOF
        }
        let available = data.len() - offset;
        let to_read = core::cmp::min(available, buf.len());
        buf[..to_read].copy_from_slice(&data[offset..offset + to_read]);
        Ok(to_read)
    }

    fn write(&self, offset: usize, buf: &[u8]) -> Result<usize, VfsError> {
        if self.node_type != VNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        let mut data = self.data.lock();
        if offset + buf.len() > data.len() {
            data.resize(offset + buf.len(), 0);
        }
        data[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn create(&self, name: &str, node_type: VNodeType) -> Result<Arc<dyn VNode>, VfsError> {
        if self.node_type != VNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        let mut children = self.children.lock();
        for (child_name, _) in children.iter() {
            if child_name == name {
                return Err(VfsError::AlreadyExists);
            }
        }
        let new_node = Arc::new(RamNode::new(node_type));
        children.push((String::from(name), new_node.clone()));
        Ok(new_node as Arc<dyn VNode>)
    }

    fn unlink(&self, name: &str) -> Result<(), VfsError> {
        if self.node_type != VNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        let mut children = self.children.lock();
        let initial_len = children.len();
        children.retain(|(child_name, _)| child_name != name);
        if children.len() < initial_len {
            Ok(())
        } else {
            Err(VfsError::NotFound)
        }
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, VfsError> {
        if self.node_type != VNodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        let children = self.children.lock();
        let mut entries = Vec::new();
        for (name, node) in children.iter() {
            entries.push(DirEntry {
                ino: node.ino,
                name: name.clone(),
                node_type: node.node_type,
            });
        }
        Ok(entries)
    }

    fn truncate(&self, size: usize) -> Result<(), VfsError> {
        if self.node_type != VNodeType::File {
            return Err(VfsError::IsADirectory);
        }
        let mut data = self.data.lock();
        data.resize(size, 0);
        Ok(())
    }
}

pub struct RamFS {
    root: Arc<RamNode>,
}

impl RamFS {
    pub fn new() -> Self {
        let root = Arc::new(RamNode::new(VNodeType::Directory));
        RamFS { root }
    }
}

impl FileSystem for RamFS {
    fn name(&self) -> &'static str {
        "ramfs"
    }

    fn root(&self) -> Arc<dyn VNode> {
        self.root.clone() as Arc<dyn VNode>
    }
}
