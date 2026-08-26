// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const LOOPBACK: Ipv4Address = Ipv4Address([127, 0, 0, 1]);
    pub const BROADCAST: Ipv4Address = Ipv4Address([255, 255, 255, 255]);
    pub const ZERO: Ipv4Address = Ipv4Address([0, 0, 0, 0]);

    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Address([a, b, c, d])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IpProtocol {
    ICMP = 1,
    TCP  = 6,
    UDP  = 17,
    Unknown(u8),
}

impl From<u8> for IpProtocol {
    fn from(val: u8) -> Self {
        match val {
            1 => IpProtocol::ICMP,
            6 => IpProtocol::TCP,
            17 => IpProtocol::UDP,
            other => IpProtocol::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ipv4Header {
    pub version_ihl: u8,
    pub tos: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags_fragment_offset: u16,
    pub ttl: u8,
    pub protocol: IpProtocol,
    pub checksum: u16,
    pub src: Ipv4Address,
    pub dst: Ipv4Address,
}

impl Ipv4Header {
    pub const MIN_SIZE: usize = 20;

    pub fn parse(buf: &[u8]) -> Option<(Self, &[u8])> {
        if buf.len() < Self::MIN_SIZE {
            return None;
        }

        let version_ihl = buf[0];
        let version = version_ihl >> 4;
        let ihl = (version_ihl & 0x0F) as usize * 4;

        if version != 4 || buf.len() < ihl || ihl < Self::MIN_SIZE {
            return None;
        }

        let tos = buf[1];
        let total_length = u16::from_be_bytes([buf[2], buf[3]]);
        let identification = u16::from_be_bytes([buf[4], buf[5]]);
        let flags_fragment_offset = u16::from_be_bytes([buf[6], buf[7]]);
        let ttl = buf[8];
        let protocol = IpProtocol::from(buf[9]);
        let checksum = u16::from_be_bytes([buf[10], buf[11]]);

        let mut src = [0u8; 4];
        let mut dst = [0u8; 4];
        src.copy_from_slice(&buf[12..16]);
        dst.copy_from_slice(&buf[16..20]);

        Some((
            Ipv4Header {
                version_ihl,
                tos,
                total_length,
                identification,
                flags_fragment_offset,
                ttl,
                protocol,
                checksum,
                src: Ipv4Address(src),
                dst: Ipv4Address(dst),
            },
            &buf[ihl..],
        ))
    }

    pub fn calculate_checksum(header_bytes: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        for chunk in header_bytes.chunks(2) {
            if chunk.len() == 2 {
                let word = u16::from_be_bytes([chunk[0], chunk[1]]);
                sum = sum.wrapping_add(word as u32);
            } else {
                let word = u16::from_be_bytes([chunk[0], 0]);
                sum = sum.wrapping_add(word as u32);
            }
        }
        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        !sum as u16
    }
}
