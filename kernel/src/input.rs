// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use crate::events::{RawHardwareEvent, InputEvent, Key, MouseButton};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyState};

pub struct InputDispatcher {
    keyboard: Keyboard<layouts::Us104Key, ScancodeSet1>,
    mouse_cycle: u8,
    mouse_packet: [u8; 3],
    mouse_state: [bool; 3], // keep track of previous button state
}

impl InputDispatcher {
    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore),
            mouse_cycle: 0,
            mouse_packet: [0; 3],
            mouse_state: [false; 3],
        }
    }

    pub fn process_event<F>(&mut self, event: RawHardwareEvent, mut on_event: F)
    where
        F: FnMut(InputEvent),
    {
        match event {
            RawHardwareEvent::KeyboardScancode(scancode) => {
                if let Ok(Some(key_event)) = self.keyboard.add_byte(scancode) {
                    let is_down = key_event.state == KeyState::Down;
                    if let Some(key) = self.keyboard.process_keyevent(key_event) {
                        let generic_key = match key {
                            DecodedKey::Unicode(character) => Key::Character(character),
                            DecodedKey::RawKey(k) => Key::Raw(k as u16),
                        };
                        
                        if is_down {
                            on_event(InputEvent::KeyDown { key: generic_key });
                        } else {
                            on_event(InputEvent::KeyUp { key: generic_key });
                        }
                    }
                }
            }
            RawHardwareEvent::MouseByte(byte) => {
                match self.mouse_cycle {
                    0 => {
                        // Wait for sync bit (bit 3 should be 1)
                        if (byte & 0x08) != 0 {
                            self.mouse_packet[0] = byte;
                            self.mouse_cycle = 1;
                        }
                    }
                    1 => {
                        self.mouse_packet[1] = byte;
                        self.mouse_cycle = 2;
                    }
                    2 => {
                        self.mouse_packet[2] = byte;
                        self.mouse_cycle = 0;
                        self.decode_mouse_packet(&mut on_event);
                    }
                    _ => self.mouse_cycle = 0,
                }
            }
        }
    }

    fn decode_mouse_packet<F>(&mut self, on_event: &mut F)
    where
        F: FnMut(InputEvent),
    {
        let state = self.mouse_packet[0];
        
        // Check for overflow
        if (state & 0xC0) != 0 {
            return; 
        }

        let mut dx = self.mouse_packet[1] as i32;
        let mut dy = self.mouse_packet[2] as i32;

        // Sign extend
        if (state & 0x10) != 0 { dx |= !0xFF; }
        if (state & 0x20) != 0 { dy |= !0xFF; }

        let left_btn = (state & 0x01) != 0;
        let right_btn = (state & 0x02) != 0;
        let middle_btn = (state & 0x04) != 0;

        if dx != 0 || dy != 0 {
            on_event(InputEvent::MouseMove { dx, dy: -dy });
        }
        
        let new_state = [left_btn, right_btn, middle_btn];
        let buttons = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];
        
        for i in 0..3 {
            if new_state[i] && !self.mouse_state[i] {
                on_event(InputEvent::MouseButtonDown { button: buttons[i] });
            } else if !new_state[i] && self.mouse_state[i] {
                on_event(InputEvent::MouseButtonUp { button: buttons[i] });
            }
        }
        
        self.mouse_state = new_state;
    }
}
