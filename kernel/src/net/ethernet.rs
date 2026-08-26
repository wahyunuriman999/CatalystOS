// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub const BROADCAST: MacAddress = MacAddress([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    pub const ZERO: MacAddress = MacAddress([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        MacAddress([a, b, c, d, e, f])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum EtherType {
    IPv4 = 0x0800,
    ARP  = 0x0806,
    IPv6 = 0x86DD,
    Unknown(u16),
}

impl From<u16> for EtherType {
    fn from(val: u16) -> Self {
        match val {
            0x0800 => EtherType::IPv4,
            0x0806 => EtherType::ARP,
            0x86DD => EtherType::IPv6,
            other => EtherType::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EthernetHeader {
    pub dst: MacAddress,
    pub src: MacAddress,
    pub ethertype: EtherType,
}

impl EthernetHeader {
    pub const SIZE: usize = 14;

    pub fn parse(buf: &[u8]) -> Option<(Self, &[u8])> {
        if buf.len() < Self::SIZE {
            return None;
        }

        let mut dst = [0u8; 6];
        let mut src = [0u8; 6];
        dst.copy_from_slice(&buf[0..6]);
        src.copy_from_slice(&buf[6..12]);

        let ethertype_raw = u16::from_be_bytes([buf[12], buf[13]]);
        let ethertype = EtherType::from(ethertype_raw);

        Some((
            EthernetHeader {
                dst: MacAddress(dst),
                src: MacAddress(src),
                ethertype,
            },
            &buf[Self::SIZE..],
        ))
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Option<usize> {
        if buf.len() < Self::SIZE {
            return None;
        }
        buf[0..6].copy_from_slice(&self.dst.0);
        buf[6..12].copy_from_slice(&self.src.0);
        let et_raw = match self.ethertype {
            EtherType::IPv4 => 0x0800,
            EtherType::ARP => 0x0806,
            EtherType::IPv6 => 0x86DD,
            EtherType::Unknown(v) => v,
        };
        buf[12..14].copy_from_slice(&et_raw.to_be_bytes());
        Some(Self::SIZE)
    }
}
