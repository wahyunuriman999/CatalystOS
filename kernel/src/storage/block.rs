// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::vec::Vec;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockError {
    OutOfBounds,
    IoError,
    ReadOnly,
    NotReady,
}

pub trait BlockDevice: Send + Sync {
    fn block_size(&self) -> usize;
    fn total_blocks(&self) -> u64;
    fn read_block(&self, block_id: u64, buf: &mut [u8]) -> Result<(), BlockError>;
    fn write_block(&self, block_id: u64, buf: &[u8]) -> Result<(), BlockError>;
}

/// In-memory RAM disk block device for testing filesystems and block caching.
pub struct RamDisk {
    block_size: usize,
    blocks: Mutex<Vec<u8>>,
}

impl RamDisk {
    pub fn new(total_blocks: usize, block_size: usize) -> Self {
        let size = total_blocks * block_size;
        let mut data = Vec::with_capacity(size);
        data.resize(size, 0);
        RamDisk {
            block_size,
            blocks: Mutex::new(data),
        }
    }
}

impl BlockDevice for RamDisk {
    fn block_size(&self) -> usize {
        self.block_size
    }

    fn total_blocks(&self) -> u64 {
        let len = self.blocks.lock().len();
        (len / self.block_size) as u64
    }

    fn read_block(&self, block_id: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        let start = (block_id as usize) * self.block_size;
        let end = start + self.block_size;
        let data = self.blocks.lock();
        if end > data.len() || buf.len() < self.block_size {
            return Err(BlockError::OutOfBounds);
        }
        buf[..self.block_size].copy_from_slice(&data[start..end]);
        Ok(())
    }

    fn write_block(&self, block_id: u64, buf: &[u8]) -> Result<(), BlockError> {
        let start = (block_id as usize) * self.block_size;
        let end = start + self.block_size;
        let mut data = self.blocks.lock();
        if end > data.len() || buf.len() < self.block_size {
            return Err(BlockError::OutOfBounds);
        }
        data[start..end].copy_from_slice(&buf[..self.block_size]);
        Ok(())
    }
}
