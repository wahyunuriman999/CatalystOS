<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Phase 2B: Generic Input API Specification

This document defines the generic input interface that bridges the hardware-specific decoder layer and the Window Manager.

## Architectural Goal
The Window Manager and any high-level UI component MUST remain completely agnostic to hardware details. They must not know about:
- IRQ numbers (e.g., IRQ 1, IRQ 12)
- Hardware ports (e.g., `0x60`, `0x64`)
- PIC or APIC implementations
- Driver-specific protocols (e.g., PS/2 scancodes or 3-byte mouse packets)
- Whether the input originated from PS/2, USB HID, or a virtual device.

## The `InputEvent` Enum

The core of the Generic Input API is the `InputEvent` enum. It represents high-level semantic actions.

```rust
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
    Raw(u16), // Fallback for special keys without a mapped char
    // Future: Enter, Backspace, ArrowUp, etc.
}
### Keyboard Layout Boundary (P2B-H3 Limitation)
**Limitation:** The current `InputDispatcher` implementation uses `pc-keyboard` with a hardcoded `Us104Key` layout to emit `Key::Character`.
**Future Architecture:** The Generic Input API must not assume a physical US layout. `InputEvent` will eventually emit semantic Virtual Keys (e.g., `VirtualKey::A`, `VirtualKey::Shift`), and a separate user-space Keyboard Layout Manager will translate these Virtual Keys into Unicode characters based on the user's localized configuration.

## Data Flow

1. **Hardware Interrupt (ISR):** Reads raw byte from port and pushes `RawHardwareEvent` to the Lock-Free `EventQueue`.
2. **Input Dispatcher (Main Loop):** 
   - Pops `RawHardwareEvent`.
   - Decodes protocol (e.g., PS/2 mouse packet state machine, Keyboard scancode mapping).
   - Generates one or more `InputEvent`s.
3. **Window Manager:**
   - Consumes `InputEvent`.
   - Updates global cursor position (`MouseMove`).
   - Routes clicks to the top-most window (`MouseButtonDown`).
   - Routes keystrokes to the focused window (`KeyDown`).

## Hardware Independence Rule
Any new input driver (e.g., USB HID) will simply push to `RawHardwareEvent` (if a decoder is used) OR bypass the decoder and generate `InputEvent` directly if the driver does protocol decoding internally, ensuring the Window Manager code remains 100% untouched when hardware changes.
