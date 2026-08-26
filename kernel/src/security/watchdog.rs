// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;

pub struct Watchdog {
    pub timeout_ticks: u32,
    pub counter: AtomicU32,
    pub tripped: bool,
}

impl Watchdog {
    pub const fn new(timeout_ticks: u32) -> Self {
        Watchdog {
            timeout_ticks,
            counter: AtomicU32::new(timeout_ticks),
            tripped: false,
        }
    }

    pub fn pet(&self) {
        self.counter.store(self.timeout_ticks, Ordering::Relaxed);
    }

    pub fn tick(&mut self) -> bool {
        let cur = self.counter.load(Ordering::Relaxed);
        if cur == 0 {
            self.tripped = true;
            true
        } else {
            self.counter.store(cur - 1, Ordering::Relaxed);
            false
        }
    }
}

pub static KERNEL_WATCHDOG: Mutex<Watchdog> = Mutex::new(Watchdog::new(100)); // 100 ticks default
