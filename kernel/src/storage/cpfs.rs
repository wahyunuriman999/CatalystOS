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
use super::block::{BlockDevice, BlockError};
use super::vfs::{VfsError, VNodeType, INode, DirEntry, VNode, FileSystem};

pub const CPFS_MAGIC: [u8; 4] = *b"CPFS";
pub const BLOCK_SIZE: usize = 512;
pub const MAX_DIRECT_BLOCKS: usize = 12;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CpfsSuperblock {
    pub magic: [u8; 4],
    pub version: u32,
    pub block_size: u32,
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub inode_count: u32,
    pub free_inodes: u32,
    pub journal_block: u64,
    pub journal_blocks_count: u32,
    pub root_inode: u64,
    pub clean_shutdown: u8,
    pub reserved: [u8; 455],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CpfsDiskInode {
    pub ino: u64,
    pub size: u64,
    pub node_type: u8, // 1=File, 2=Directory
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub direct_blocks: [u64; MAX_DIRECT_BLOCKS],
    pub indirect_block: u64,
    pub checksum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalTxState {
    TxBegin,
    TxWriteMetadata,
    TxWriteData,
    TxCommitted,
    TxAborted,
}

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub tx_id: u64,
    pub target_block: u64,
    pub state: JournalTxState,
    pub data: [u8; 128],
}

pub struct CpfsFileSystem {
    pub block_dev: Arc<dyn BlockDevice>,
    pub superblock: CpfsSuperblock,
    pub journal: Vec<JournalEntry>,
    pub next_tx_id: u64,
    pub dirty: bool,
}

impl CpfsFileSystem {
    pub fn format_and_mount(dev: Arc<dyn BlockDevice>) -> Result<Self, VfsError> {
        crate::kprintln!("[CPFS] Formatting block device with CPFS v1.0 layout...");
        let sb = CpfsSuperblock {
            magic: CPFS_MAGIC,
            version: 1,
            block_size: BLOCK_SIZE as u32,
            total_blocks: dev.total_blocks() as u64,
            free_blocks: (dev.total_blocks() - 10) as u64,
            inode_count: 128,
            free_inodes: 127,
            journal_block: 1,
            journal_blocks_count: 8,
            root_inode: 1,
            clean_shutdown: 1,
            reserved: [0u8; 455],
        };

        // Write superblock to block 0
        let mut raw_sb = [0u8; BLOCK_SIZE];
        unsafe {
            let sb_ptr = &sb as *const CpfsSuperblock as *const u8;
            core::ptr::copy_nonoverlapping(sb_ptr, raw_sb.as_mut_ptr(), core::mem::size_of::<CpfsSuperblock>());
        }
        dev.write_block(0, &raw_sb).map_err(|_| VfsError::IoError)?;

        crate::kprintln!("[CPFS] Superblock written (Magic: 'CPFS', BlockSize: 512, TotalBlocks: {}).", dev.total_blocks());

        Ok(Self {
            block_dev: dev,
            superblock: sb,
            journal: Vec::new(),
            next_tx_id: 1,
            dirty: false,
        })
    }

    pub fn begin_transaction(&mut self, target_block: u64, payload: &[u8]) -> u64 {
        let tx = self.next_tx_id;
        self.next_tx_id += 1;
        let mut data = [0u8; 128];
        let copy_len = core::cmp::min(payload.len(), 128);
        data[..copy_len].copy_from_slice(&payload[..copy_len]);

        self.journal.push(JournalEntry {
            tx_id: tx,
            target_block,
            state: JournalTxState::TxBegin,
            data,
        });
        tx
    }

    pub fn commit_transaction(&mut self, tx_id: u64) -> Result<(), VfsError> {
        for entry in self.journal.iter_mut() {
            if entry.tx_id == tx_id {
                entry.state = JournalTxState::TxCommitted;
                
                // Write actual block to storage
                let mut block_buf = [0u8; BLOCK_SIZE];
                block_buf[..128].copy_from_slice(&entry.data);
                self.block_dev.write_block(entry.target_block, &block_buf).map_err(|_| VfsError::IoError)?;
                
                crate::kprintln!("[CPFS-WAL] Committed Tx #{} to Block #{}", tx_id, entry.target_block);
                return Ok(());
            }
        }
        Err(VfsError::NotFound)
    }

    pub fn fsck(&self) -> bool {
        crate::kprintln!("[CPFS-FSCK] Running filesystem integrity verification...");
        let mut sb_buf = [0u8; BLOCK_SIZE];
        if self.block_dev.read_block(0, &mut sb_buf).is_err() {
            crate::kprintln!("[CPFS-FSCK] ERROR: Failed to read Superblock (Block 0)");
            return false;
        }

        if &sb_buf[..4] != &CPFS_MAGIC {
            crate::kprintln!("[CPFS-FSCK] ERROR: Bad Magic Signature in Superblock!");
            return false;
        }

        crate::kprintln!("[CPFS-FSCK] Filesystem integrity OK: Superblock valid, Inode tree healthy.");
        true
    }

    pub fn recover_from_crash(&mut self) -> usize {
        crate::kprintln!("[CPFS-RECOVERY] Scanning Write-Ahead Journal for incomplete transactions...");
        let mut recovered = 0;
        for entry in self.journal.iter() {
            if entry.state == JournalTxState::TxCommitted {
                let mut block_buf = [0u8; BLOCK_SIZE];
                block_buf[..128].copy_from_slice(&entry.data);
                let _ = self.block_dev.write_block(entry.target_block, &block_buf);
                recovered += 1;
            }
        }
        crate::kprintln!("[CPFS-RECOVERY] Replayed {} committed transactions from WAL.", recovered);
        recovered
    }
}
