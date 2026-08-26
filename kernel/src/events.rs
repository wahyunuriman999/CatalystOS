// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;

#[derive(Debug, Clone, Copy)]
pub enum RawHardwareEvent {
    KeyboardScancode(u8),
    MouseByte(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    KeyDown { key: Key },
    KeyUp { key: Key },
    MouseMove { dx: i32, dy: i32 },
    MouseButtonDown { button: MouseButton },
    MouseButtonUp { button: MouseButton },
    MouseScroll { delta: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Character(char),
    Raw(u16),
}

const QUEUE_SIZE: usize = 256;

pub struct EventQueue {
    buffer: UnsafeCell<[Option<RawHardwareEvent>; QUEUE_SIZE]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    pub push_count: AtomicUsize,
    pub drop_count: AtomicUsize,
    pub pop_count: AtomicUsize,
}

unsafe impl Sync for EventQueue {}

impl EventQueue {
    pub const fn new() -> Self {
        const INIT: Option<RawHardwareEvent> = None;
        Self {
            buffer: UnsafeCell::new([INIT; QUEUE_SIZE]),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            push_count: AtomicUsize::new(0),
            drop_count: AtomicUsize::new(0),
            pop_count: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, event: RawHardwareEvent) -> Result<(), &'static str> {
        let head = self.head.load(Ordering::Relaxed);
        let next_head = (head + 1) % QUEUE_SIZE;
        
        if next_head == self.tail.load(Ordering::Acquire) {
            self.drop_count.fetch_add(1, Ordering::Relaxed);
            return Err("Event queue full");
        }
        
        unsafe {
            let buf = &mut *self.buffer.get();
            buf[head] = Some(event);
        }
        self.head.store(next_head, Ordering::Release);
        self.push_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn pop(&self) -> Option<RawHardwareEvent> {
        let tail = self.tail.load(Ordering::Relaxed);
        
        if tail == self.head.load(Ordering::Acquire) {
            return None; // Empty
        }
        
        let event = unsafe {
            let buf = &mut *self.buffer.get();
            buf[tail].take()
        };
        self.tail.store((tail + 1) % QUEUE_SIZE, Ordering::Release);
        self.pop_count.fetch_add(1, Ordering::Relaxed);
        event
    }
}

pub static EVENT_QUEUE: EventQueue = EventQueue::new();

pub fn push_event(event: RawHardwareEvent) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let _ = EVENT_QUEUE.push(event);
    });
}

pub fn pop_event() -> Option<RawHardwareEvent> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        EVENT_QUEUE.pop()
    })
}

pub fn peek_event() -> Option<RawHardwareEvent> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let tail = EVENT_QUEUE.tail.load(core::sync::atomic::Ordering::Relaxed);
        if tail == EVENT_QUEUE.head.load(core::sync::atomic::Ordering::Acquire) {
            None
        } else {
            Some(RawHardwareEvent::KeyboardScancode(0)) // dummy value, we just need Some
        }
    })
}
