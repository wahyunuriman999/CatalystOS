// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::string::String;
use alloc::vec::Vec;
use crate::storage::vfs::{vfs_open, O_CREAT, O_WRONLY, O_TRUNC};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageError {
    InvalidMagic,
    CorruptedHeader,
    ChecksumMismatch,
    IoError,
}

#[derive(Debug, Clone)]
pub struct PackageHeader {
    pub magic: [u8; 4], // 'CPKG'
    pub version: u32,
    pub name: String,
    pub payload_size: usize,
    pub checksum: u32,
}

impl PackageHeader {
    pub const MAGIC: [u8; 4] = [b'C', b'P', b'K', b'G'];

    pub fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), PackageError> {
        if bytes.len() < 32 {
            return Err(PackageError::CorruptedHeader);
        }

        if &bytes[0..4] != Self::MAGIC {
            return Err(PackageError::InvalidMagic);
        }

        let version = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let payload_size = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let checksum = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        // Name length in byte 16
        let name_len = bytes[16] as usize;
        if bytes.len() < 32 + payload_size || name_len > 15 {
            return Err(PackageError::CorruptedHeader);
        }

        let name_str = core::str::from_utf8(&bytes[17..17 + name_len])
            .map_err(|_| PackageError::CorruptedHeader)?;

        let payload = &bytes[32..32 + payload_size];
        
        // Verify Adler-32 / simple sum checksum
        let mut calc_csum: u32 = 0;
        for &b in payload {
            calc_csum = calc_csum.wrapping_add(b as u32);
        }

        if calc_csum != checksum {
            return Err(PackageError::ChecksumMismatch);
        }

        Ok((
            PackageHeader {
                magic: Self::MAGIC,
                version,
                name: String::from(name_str),
                payload_size,
                checksum,
            },
            payload,
        ))
    }

    pub fn serialize(name: &str, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 + payload.len());
        out.extend_from_slice(&Self::MAGIC);
        out.extend_from_slice(&1u32.to_be_bytes()); // version 1
        out.extend_from_slice(&(payload.len() as u32).to_be_bytes());

        let mut calc_csum: u32 = 0;
        for &b in payload {
            calc_csum = calc_csum.wrapping_add(b as u32);
        }
        out.extend_from_slice(&calc_csum.to_be_bytes());

        let name_bytes = name.as_bytes();
        let name_len = core::cmp::min(name_bytes.len(), 15);
        out.push(name_len as u8);
        out.extend_from_slice(&name_bytes[..name_len]);
        while out.len() < 32 {
            out.push(0);
        }

        out.extend_from_slice(payload);
        out
    }
}

/// Atomically install a package payload into the VFS system directory.
pub fn install_package(pkg_bytes: &[u8]) -> Result<String, PackageError> {
    let (header, payload) = PackageHeader::parse(pkg_bytes)?;
    
    let dest_path = alloc::format!("/bin/{}", header.name);
    let file = vfs_open(&dest_path, O_CREAT | O_WRONLY | O_TRUNC)
        .map_err(|_| PackageError::IoError)?;
        
    file.write(0, payload).map_err(|_| PackageError::IoError)?;
    
    crate::kprintln!("[PKG] Installed package '{}' ({} bytes) to {}", header.name, payload.len(), dest_path);
    Ok(header.name)
}
