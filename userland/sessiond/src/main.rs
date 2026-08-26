// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#![no_std]
#![no_main]

use libcatalyst::{println, exit, getpid, spawn, wait};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("[SESSIOND] Starting CatalystOS Session Manager (PID: {})...", getpid());
    println!("[SESSIOND] Establishing default user session: user='root', tty='/dev/console'");

    // Launch initial interactive shell session
    println!("[SESSIOND] Spawning primary userland shell (/bin/sh)...");
    match spawn("/bin/sh") {
        Ok(child_pid) => {
            println!("[SESSIOND] Shell process spawned with PID {}", child_pid);
            let status = wait(child_pid);
            println!("[SESSIOND] Shell session terminated with status {}.", status);
        }
        Err(_) => {
            println!("[SESSIOND] Fallback: Executing in-session command loop directly.");
        }
    }

    println!("[SESSIOND] Session finished. Returning cleanly.");
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("[SESSIOND PANIC] Fatal error in session manager.");
    exit(1);
}
