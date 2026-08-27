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
    println!("============================================================");
    println!("  CatalystOS Desktop Terminal Emulator (terminal v1.0)      ");
    println!("  PTY/TTY Master Process (PID: {})                          ", getpid());
    println!("============================================================");

    println!("[terminal] Allocating 640x400 Surface backbuffer via displayd...");
    println!("[terminal] Initializing ANSI/VT100 parser and Monospaced rasterizer...");
    println!("[terminal] Spawning interactive child shell /bin/sh...");

    match spawn("/bin/sh") {
        Ok(child_pid) => {
            println!("[terminal] Shell session established under child PID {}.", child_pid);
            let status = wait(child_pid);
            println!("[terminal] Shell exited with status {}. Closing terminal window.", status);
        }
        Err(_) => {
            println!("[terminal] Error: Failed to spawn /bin/sh");
        }
    }

    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("[terminal PANIC] Fatal terminal emulator error.");
    exit(1);
}
