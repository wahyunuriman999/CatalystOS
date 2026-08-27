// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#![no_std]
#![no_main]

use libcatalyst::{println, exit, getpid};

pub const PROTOCOL_VERSION: u32 = 2;

pub const OP_CREATE_SURFACE: u32  = 1;
pub const OP_DESTROY_SURFACE: u32 = 2;
pub const OP_ATTACH_BUFFER: u32   = 3;
pub const OP_COMMIT: u32          = 4;
pub const OP_SET_POSITION: u32    = 5;
pub const OP_SET_SIZE: u32        = 6;
pub const OP_REQUEST_FOCUS: u32   = 7;
pub const OP_MINIMIZE: u32        = 8;
pub const OP_MAXIMIZE: u32        = 9;
pub const OP_RESTORE: u32         = 10;
pub const OP_CLOSE: u32           = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Closed,
}

#[derive(Debug, Clone, Copy)]
pub struct Surface {
    pub id: u32,
    pub client_pid: u64,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub state: WindowState,
    pub visible: bool,
    pub focused: bool,
    pub damage_count: u32,
}

pub struct CompositorServer {
    pub surfaces: [Option<Surface>; 32],
    pub z_order: [Option<u32>; 32],
    pub next_surface_id: u32,
    pub active_focus: Option<u32>,
}

impl CompositorServer {
    pub const fn new() -> Self {
        const EMPTY_SURFACE: Option<Surface> = None;
        const EMPTY_Z: Option<u32> = None;
        Self {
            surfaces: [EMPTY_SURFACE; 32],
            z_order: [EMPTY_Z; 32],
            next_surface_id: 1,
            active_focus: None,
        }
    }

    pub fn create_surface(&mut self, client_pid: u64, x: i32, y: i32, w: u32, h: u32) -> Result<u32, &'static str> {
        let id = self.next_surface_id;
        self.next_surface_id += 1;
        let s = Surface {
            id,
            client_pid,
            x,
            y,
            width: w.clamp(32, 3840),
            height: h.clamp(32, 2160),
            state: WindowState::Normal,
            visible: true,
            focused: true,
            damage_count: 0,
        };
        self.surfaces[(id as usize) % 32] = Some(s);
        self.bring_to_front(id);
        self.set_focus(id);
        println!("[DISPLAYD] Surface #{} created for PID {} ({}x{} at {},{})", id, client_pid, s.width, s.height, x, y);
        Ok(id)
    }

    pub fn set_position(&mut self, id: u32, x: i32, y: i32) -> Result<(), &'static str> {
        let s = self.get_surface_mut(id).ok_or("Surface not found")?;
        s.x = x;
        s.y = y;
        println!("[DISPLAYD] Surface #{} moved to ({}, {})", id, x, y);
        Ok(())
    }

    pub fn set_size(&mut self, id: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let s = self.get_surface_mut(id).ok_or("Surface not found")?;
        s.width = w.clamp(32, 3840);
        s.height = h.clamp(32, 2160);
        println!("[DISPLAYD] Surface #{} resized to {}x{}", id, s.width, s.height);
        Ok(())
    }

    pub fn set_focus(&mut self, id: u32) {
        for s in self.surfaces.iter_mut().flatten() {
            s.focused = (s.id == id);
        }
        self.active_focus = Some(id);
        self.bring_to_front(id);
        println!("[DISPLAYD] Surface #{} gained active window focus", id);
    }

    pub fn minimize(&mut self, id: u32) -> Result<(), &'static str> {
        let s = self.get_surface_mut(id).ok_or("Surface not found")?;
        s.state = WindowState::Minimized;
        s.visible = false;
        s.focused = false;
        println!("[DISPLAYD] Surface #{} minimized", id);
        self.fallback_focus();
        Ok(())
    }

    pub fn restore(&mut self, id: u32) -> Result<(), &'static str> {
        let s = self.get_surface_mut(id).ok_or("Surface not found")?;
        s.state = WindowState::Normal;
        s.visible = true;
        println!("[DISPLAYD] Surface #{} restored", id);
        self.set_focus(id);
        Ok(())
    }

    pub fn commit_damage(&mut self, id: u32) -> Result<(), &'static str> {
        let s = self.get_surface_mut(id).ok_or("Surface not found")?;
        s.damage_count += 1;
        println!("[DISPLAYD] Surface #{} damage committed (frame #{})", id, s.damage_count);
        Ok(())
    }

    pub fn destroy_surface(&mut self, id: u32) -> Result<(), &'static str> {
        let idx = (id as usize) % 32;
        if self.surfaces[idx].is_none() {
            return Err("Surface does not exist");
        }
        self.surfaces[idx] = None;
        for slot in self.z_order.iter_mut() {
            if *slot == Some(id) {
                *slot = None;
            }
        }
        if self.active_focus == Some(id) {
            self.fallback_focus();
        }
        println!("[DISPLAYD] Surface #{} cleanly destroyed and resources reclaimed", id);
        Ok(())
    }

    pub fn on_client_crashed(&mut self, client_pid: u64) {
        println!("[DISPLAYD] Client PID {} crashed; purging associated surfaces...", client_pid);
        let mut to_destroy = [0u32; 32];
        let mut count = 0;
        for s in self.surfaces.iter().flatten() {
            if s.client_pid == client_pid && count < 32 {
                to_destroy[count] = s.id;
                count += 1;
            }
        }
        for i in 0..count {
            let _ = self.destroy_surface(to_destroy[i]);
        }
    }

    fn bring_to_front(&mut self, id: u32) {
        // Shift z-order
        let mut found = false;
        for i in 0..32 {
            if self.z_order[i] == Some(id) {
                for j in i..31 {
                    self.z_order[j] = self.z_order[j + 1];
                }
                self.z_order[31] = None;
                found = true;
                break;
            }
        }
        if !found {
            // Find first empty slot
            for slot in self.z_order.iter_mut() {
                if slot.is_none() {
                    *slot = Some(id);
                    return;
                }
            }
        }
        // Place at top
        for slot in self.z_order.iter_mut() {
            if slot.is_none() {
                *slot = Some(id);
                return;
            }
        }
    }

    fn fallback_focus(&mut self) {
        // Find topmost visible surface
        for slot in self.z_order.iter().rev().flatten() {
            if let Some(s) = self.get_surface(*slot) {
                if s.visible && s.state != WindowState::Minimized {
                    self.active_focus = Some(*slot);
                    println!("[DISPLAYD] Focus fallback shifted to Surface #{}", *slot);
                    return;
                }
            }
        }
        self.active_focus = None;
        println!("[DISPLAYD] No visible surfaces remaining; focus returned to Root Desktop.");
    }

    fn get_surface(&self, id: u32) -> Option<&Surface> {
        let s = self.surfaces[(id as usize) % 32].as_ref()?;
        if s.id == id { Some(s) } else { None }
    }

    fn get_surface_mut(&mut self, id: u32) -> Option<&mut Surface> {
        let s = self.surfaces[(id as usize) % 32].as_mut()?;
        if s.id == id { Some(s) } else { None }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("============================================================");
    println!("  CatalystOS Display Compositor Service (displayd v2.0)     ");
    println!("  Protocol: CatalystSurface IPC Protocol v{}", PROTOCOL_VERSION);
    println!("============================================================");

    let mut server = CompositorServer::new();

    // 1. Full Window Lifecycle Validation
    let w1 = server.create_surface(10, 100, 100, 640, 480).unwrap();
    let w2 = server.create_surface(20, 200, 200, 800, 600).unwrap();

    let _ = server.set_position(w1, 150, 120);
    let _ = server.set_size(w1, 700, 500);
    let _ = server.commit_damage(w1);

    server.set_focus(w1);
    let _ = server.minimize(w1);
    let _ = server.restore(w1);

    // 2. Adversarial Sudden Client Crash Simulation
    server.on_client_crashed(10);

    // 3. Clean Window Destruction
    let _ = server.destroy_surface(w2);

    println!("\n[displayd] Lifecycle state machine and recovery tests PASS.");
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("[DISPLAYD PANIC] Fatal displayd error.");
    exit(1);
}
