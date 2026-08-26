// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libcatalyst::{println, exit, getpid};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let pid = getpid();
    println!("Hello from Catalyst Userland (PID: {})!", pid);
    println!("libcatalyst ABI verification successful.");
    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
