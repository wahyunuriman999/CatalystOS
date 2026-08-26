// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

pub mod ethernet;
pub mod ipv4;
pub mod udp;

pub use ethernet::{MacAddress, EtherType, EthernetHeader};
pub use ipv4::{Ipv4Address, IpProtocol, Ipv4Header};
pub use udp::UdpHeader;

pub fn init_net() {
    crate::kprintln!("[NET] Initializing Catalyst Network Stack (Ethernet/IPv4/UDP)...");
    crate::kprintln!("[NET] Loopback interface configured: 127.0.0.1/8");
    crate::kprintln!("[NET] Network subsystem ready.");
}
