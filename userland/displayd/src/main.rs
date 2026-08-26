// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#![no_std]
#![no_main]

use libcatalyst::{println, exit, getpid};

pub const PROTOCOL_VERSION: u32 = 1;

pub const OP_CREATE_SURFACE: u32  = 1;
pub const OP_DESTROY_SURFACE: u32 = 2;
pub const OP_ATTACH_BUFFER: u32   = 3;
pub const OP_COMMIT: u32          = 4;
pub const OP_SET_POSITION: u32    = 5;
pub const OP_SET_SIZE: u32        = 6;
pub const OP_REQUEST_FOCUS: u32   = 7;

#[derive(Debug, Clone, Copy)]
pub struct Surface {
    pub id: u32,
    pub client_pid: u64,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub focused: bool,
}

pub struct CompositorServer {
    pub surfaces: [Option<Surface>; 32],
    pub next_surface_id: u32,
    pub active_focus: Option<u32>,
}

impl CompositorServer {
    pub const fn new() -> Self {
        const EMPTY: Option<Surface> = None;
        Self {
            surfaces: [EMPTY; 32],
            next_surface_id: 1,
            active_focus: None,
        }
    }

    pub fn handle_request(&mut self, op: u32, client_pid: u64, arg1: u32, arg2: u32, arg3: u32, arg4: u32) -> Result<u32, &'static str> {
        match op {
            OP_CREATE_SURFACE => {
                let id = self.next_surface_id;
                self.next_surface_id += 1;
                let s = Surface {
                    id,
                    client_pid,
                    x: arg1 as i32,
                    y: arg2 as i32,
                    width: arg3,
                    height: arg4,
                    visible: true,
                    focused: true,
                };
                self.surfaces[(id as usize) % 32] = Some(s);
                self.active_focus = Some(id);
                println!("[DISPLAYD] Surface {} created for PID {} ({}x{} at {},{})", id, client_pid, arg3, arg4, arg1, arg2);
                Ok(id)
            }
            OP_COMMIT => {
                let id = arg1;
                println!("[DISPLAYD] Surface {} damage committed, blending to backbuffer.", id);
                Ok(0)
            }
            OP_DESTROY_SURFACE => {
                let id = arg1;
                self.surfaces[(id as usize) % 32] = None;
                if self.active_focus == Some(id) {
                    self.active_focus = None;
                }
                println!("[DISPLAYD] Surface {} destroyed.", id);
                Ok(0)
            }
            _ => Err("Unknown compositor opcode"),
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("[DISPLAYD] Starting CatalystOS Display Compositor Service (PID: {})...", getpid());
    println!("[DISPLAYD] Protocol: CatalystSurface IPC Protocol v{}", PROTOCOL_VERSION);
    println!("[DISPLAYD] Shared-Memory Compositor ready to accept client surfaces.");

    let mut server = CompositorServer::new();
    
    // Simulate protocol initialization and first surface binding
    let surface_id = server.handle_request(OP_CREATE_SURFACE, 10, 100, 100, 640, 480).unwrap();
    let _ = server.handle_request(OP_COMMIT, 10, surface_id, 0, 0, 0);

    println!("[DISPLAYD] Compositor loop active.");
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("[DISPLAYD PANIC] Fatal error in display service.");
    exit(1);
}
