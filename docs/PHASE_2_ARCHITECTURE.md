<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# CATALYST OS — PHASE 2 ARCHITECTURE (FINALIZED)

This document establishes the architectural foundation for Phase 2, transitioning Catalyst OS from a static framebuffer to a genuine interactive operating system. 

## 1. Interrupt Flow (Phase 2A)

The interrupt architecture ensures that hardware events are handled safely without causing triple faults or corrupting the kernel state.

```text
Hardware Event (e.g., Timer, Keyboard)
       ↓
CPU Interrupt (IRQ)
       ↓
IDT (Interrupt Descriptor Table) Lookup
       ↓
Interrupt Service Routine (ISR) Stub (Assembly)
       ↓
Saves CPU Registers to Stack (InterruptStackFrame)
       ↓
Rust Interrupt Handler (e.g., `timer_handler`)
       ↓
Acknowledge PIC/APIC (EOI)
       ↓
Restore Registers (IRETQ)
```

**Constraints:**
- Interrupts (`sti`) are ONLY enabled after the GDT, IDT, and PIC are fully configured.
- Double faults must use a separate Interrupt Stack Table (IST) to prevent triple faults on stack overflow.
- **Interrupt Controller Architecture:** The initial implementation uses the Legacy PIC for QEMU compatibility, but the architecture abstracts this to support Local APIC, IOAPIC, and SMP in the future.

## 2. Input Flow (Phase 2B)

**Minimize Interrupt Handler Work:** Interrupt handlers perform only the minimum hardware-facing work required.

```text
Hardware IRQ
       ↓
Read device byte/status
       ↓
Push raw hardware event/data to Interrupt-Safe Ring Buffer
       ↓
Acknowledge interrupt
       ↓
Return from interrupt (IRETQ)
```

**Input Worker / Dispatcher:**
The heavy lifting is done outside the ISR. An Input Worker reads the raw data, decodes the hardware-specific state (e.g., PS/2 packet assembly, keyboard scancode mapping), generates a generic `InputEvent`, and dispatches it to the Window Manager.

## 3. Interrupt-Safe Event Queue

**Invariant:** A blocking mutex (`spin::Mutex`) MUST NOT be used by an ISR when the interrupted execution context may already hold the same lock.

To safely transfer data from the Interrupt Context to the Main Kernel Context, we use an **Interrupt-Safe Bounded Ring Buffer**.

```text
Keyboard IRQ ──┐
               │
Mouse IRQ ─────┤
               ↓
     Interrupt-Safe Ring Buffer (Atomic head/tail)
               ↓
        Input Worker/Dispatcher
```

## 4. Window Lifecycle (Phase 2C)

Windows are logical abstractions, not just hardcoded rectangles in a `desktop.rs` array.

**States:** `Created -> Mapped (Visible) -> Focused / Unfocused -> Unmapped -> Destroyed`

## 5. Scheduler Design & Preemptive Invariant (Phase 2D)

**Invariant:** Catalyst OS must not claim multitasking merely because multiple render functions execute in one loop. Real multitasking requires scheduler-driven task switching. An infinite loop in Task A must NOT prevent Task B from executing.

```text
Timer Interrupt (IRQ 0)
       ↓
Saves Current Task's CPU Context
       ↓
Calls `scheduler::schedule()`
       ↓
Selects Next Task from `Runnable Queue` (Round Robin)
       ↓
Restores Next Task's CPU Context
       ↓
IRETQ (Jumps to Next Task)
```

## 6. Canonical CPU Context ABI

The scheduler explicitly distinguishes the `Saved CPU Context` from the `Interrupt Return Frame`. There is ONE canonical architecture-specific layout for context switching.

```rust
pub struct CpuContext {
    // General-purpose registers (R15-R8, RDI, RSI, RBP, RBX, RDX, RCX, RAX)
    // Architecture-specific state
}

// Provided by x86_64 crate:
pub struct InterruptStackFrame {
    pub instruction_pointer: VirtAddr, // RIP
    pub code_segment: u64,             // CS
    pub cpu_flags: u64,                // RFLAGS
    pub stack_pointer: VirtAddr,       // RSP
    pub stack_segment: u64,            // SS
}
```

## 7. Distinguish Process from Thread

Address spaces are owned by Processes, not directly by Threads.

```text
Process
├── Process ID
├── AddressSpace (Page Table Root)
└── Threads
    ├── Thread A
    ├── Thread B
```

Thread scheduling within the same process does not require changing CR3. Context switching to a thread in a different process requires an address-space switch.

## 8. Address Space Abstraction (Phase 2E)

We do not hardcode CR3 directly into the scheduler. We use an abstraction:

```text
AddressSpace
├── Root Page Table
├── User Mappings
├── Kernel Mappings
└── Architecture-specific metadata
```
The `x86_64` implementation maps the `AddressSpace` to page tables and `CR3`.

## 9. Memory Isolation Invariant

**Invariant:** A user process must never be able to directly access another process's user memory or protected kernel memory.

Enforced via Virtual Address Spaces, Page Permissions (User/Supervisor), and Kernel/User Mode Separation. Invalid accesses result in controlled Page Fault exceptions.

## 10. Syscall Architecture

```text
Application (Ring 3) -> SYSCALL -> Syscall Handler (Ring 0) -> Kernel Service -> SYSRET -> Application (Ring 3)
```

**Implementation Requirements (Future):**
- Syscall MSR configuration (STAR, LSTAR, FMASK, GS/TSS).
- Syscall ABI and register argument conventions.
- Kernel stack switching and user pointer validation.

## 11. Phase 2A Scope & Verification

**Scope:** GDT/TSS, IDT, Exceptions, Interrupt Controller Abstraction, Timer IRQ, Keyboard/Mouse IRQ foundation, Interrupt-safe event transport, Safe interrupt enable.

**Verification:**
- **Exception Safety:** Divide-by-zero handled, double faults use IST, no triple faults.
- **Interrupt Safety:** IDT initialized before STI, timer increments, keyboard/mouse received, EOI issued.
- **Concurrency Safety:** ISR does not acquire blocking locks, transport does not deadlock.
- **Regression:** Framebuffer continues working.
