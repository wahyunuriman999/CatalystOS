<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# 📊 CATALYST OS — HARDWARE EVIDENCE MATRIX (PHASE H)

| Target Platform | Firmware / Arch | Discovery | Driver Binding | Input (Kbd/Mouse) | Display Engine | Storage (CPFS) | Power (ACPI S5) | Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **QEMU Reference (TCG/KVM)** | BIOS / x86_64 | 🟢 VERIFIED | 🟢 VERIFIED | 🟢 VERIFIED | 🟢 VERIFIED | 🟢 VERIFIED | 🟢 VERIFIED | 🟣 RUNTIME PASS |
| **Target #1: x86_64 PC (Legacy BIOS/MBR)** | BIOS (Legacy) | 🟡 STAGED | 🟡 STAGED | 🟡 STAGED | 🟡 STAGED | 🟡 STAGED | 🟡 STAGED | 🔵 READY FOR FLASH |
| **Target #2: x86_64 Laptop (UEFI/GOP)** | UEFI 2.x | ⏳ PLANNED | ⏳ PLANNED | ⏳ PLANNED | ⏳ PLANNED | ⏳ PLANNED | ⏳ PLANNED | ⚪ QUEUED |

---

## 🎯 Four Phase H Reality Gates

### 1. Gate H1 — Boot Reality
- [x] Bootloader Stage 1/2 loads kernel into Ring 0.
- [x] Physical memory map parsed from firmware (`MemoryRegions`).
- [x] Dynamic ACPI discovery probes `RSDP`, `XSDT`, `FADT`, `MADT`.
- [x] Unified Device Tree enumerates root system nodes.
- [x] Userspace `init` daemon spawned in Ring 3.

### 2. Gate H2 — Input Reality
- [x] Physical keyboard interrupt decoding (Scancode Set 1/2).
- [x] `inputd` daemon routes keystrokes to focused window.
- [x] Interactive Terminal session executes shell commands: `whoami`, `pwd`, `ls`, `mkdir`, `write`, `cat`.

### 3. Gate H3 — Storage Reality
- [x] Discovered block device formatted and mounted with CPFS v1.0.
- [x] Write-Ahead Journaling (WAL) records transactions before committing.
- [x] `fsck` validates superblock magic and inode structures.
- [x] Catastrophic Crash Recovery: Replays committed transactions on next boot.

### 4. Gate H4 — Desktop Reality
- [x] Physical mouse packet stream (3-byte packets) decoded into $(dx, dy)$ and button clicks.
- [x] `displayd` compositor manages window lifecycle: Create, Move, Resize, Focus, Minimize, Restore, Close.
- [x] Capability-secured clipboard daemon (`clipboardd`) enforces Zero Ambient Authority.
- [x] Client crash mitigation: Dead processes do not crash compositor or kernel.
