// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#![no_std]
#![no_main]

use libcatalyst::{println, exit, getpid};

pub const CAP_CLIPBOARD_READ: u8  = 1 << 5;
pub const CAP_CLIPBOARD_WRITE: u8 = 1 << 6;

pub struct ClipboardService {
    buffer: [u8; 1024],
    len: usize,
    mime_type: &'static str,
}

impl ClipboardService {
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; 1024],
            len: 0,
            mime_type: "text/plain",
        }
    }

    pub fn copy(&mut self, data: &[u8], mime: &'static str, client_caps: u8) -> Result<(), &'static str> {
        if client_caps & CAP_CLIPBOARD_WRITE == 0 {
            return Err("Permission Denied: Missing CAP_CLIPBOARD_WRITE");
        }
        let copy_len = core::cmp::min(data.len(), 1024);
        self.buffer[..copy_len].copy_from_slice(&data[..copy_len]);
        self.len = copy_len;
        self.mime_type = mime;
        println!("[CLIPBOARDD] Copied {} bytes ({}) to secure clipboard buffer.", copy_len, mime);
        Ok(())
    }

    pub fn paste(&self, client_caps: u8) -> Result<(&[u8], &'static str), &'static str> {
        if client_caps & CAP_CLIPBOARD_READ == 0 {
            return Err("Permission Denied: Missing CAP_CLIPBOARD_READ");
        }
        Ok((&self.buffer[..self.len], self.mime_type))
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("[CLIPBOARDD] Starting Capability-Secured Clipboard Service (PID: {})...", getpid());
    println!("[CLIPBOARDD] Enforcing Zero Ambient Authority on clipboard access.");

    let mut clip = ClipboardService::new();

    // 1. Simulate Application A Copying Data with Write Capability
    let app_a_caps = CAP_CLIPBOARD_WRITE;
    let _ = clip.copy(b"CATALYST_OS_DESKTOP_SECURE_TOKEN", "text/plain", app_a_caps);

    // 2. Simulate Application B Pasting Data with Read Capability
    let app_b_caps = CAP_CLIPBOARD_READ;
    match clip.paste(app_b_caps) {
        Ok((data, mime)) => {
            if let Ok(text) = core::str::from_utf8(data) {
                println!("[CLIPBOARDD] App B successfully pasted: '{}' ({})", text, mime);
            }
        }
        Err(e) => println!("[CLIPBOARDD] Paste failed: {}", e),
    }

    // 3. Simulate Malicious App C trying to spy without Read Capability
    let rogue_app_caps = 0;
    assert!(clip.paste(rogue_app_caps).is_err());
    println!("[CLIPBOARDD] Blocked unauthorized clipboard sniff attempt from unprivileged App C.");

    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("[CLIPBOARDD PANIC] Fatal clipboard service error.");
    exit(1);
}
