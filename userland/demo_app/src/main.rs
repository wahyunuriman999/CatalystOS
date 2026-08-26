// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#![no_std]
#![no_main]

use libcatalyst::{println, exit, getpid};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let pid = getpid();
    println!("==========================================");
    println!("  CatalystOS Demo GUI Application (PID: {})  ", pid);
    println!("==========================================");

    println!("[DEMO_APP] Connecting to displayd IPC endpoint...");
    println!("[DEMO_APP] Requesting surface: 400x300, title='Catalyst Calculator'");
    println!("[DEMO_APP] Obtained shared-memory surface capability (CAP_SHM_WRITE).");
    
    // Simulate rendering 400x300 canvas
    println!("[DEMO_APP] Rendering UI controls into surface backbuffer...");
    println!("[DEMO_APP] Committing frame damage rect [0, 0, 400, 300] to compositor.");
    
    // Simulate receiving click event
    println!("[DEMO_APP] Received InputEvent::PointerMove at (150, 120)");
    println!("[DEMO_APP] Received InputEvent::ButtonDown(Button 1)");
    println!("[DEMO_APP] Button '7' clicked -> Display updated to '7'");
    
    println!("[DEMO_APP] Application execution successful. Exiting cleanly.");
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("[DEMO_APP PANIC] Unrecoverable application error.");
    exit(1);
}
