<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# 🚀 PHASE H: REAL HARDWARE BOOT VALIDATION PROCEDURE

This document details the exact execution protocol to boot CatalystOS on physical hardware without emulators.

---

## 💾 1. Binary Image Artifact
- **Target Image:** `target/x86_64-catalyst/debug/catalyst-kernel.img` (10,721 KB)
- **Format:** Raw bootable hybrid MBR/BIOS disk image with embedded bootloader and 12 userland binaries.

---

## 🔌 2. Flashing to Physical USB Flash Drive

### On Windows:
```powershell
# Identify USB Drive Letter / Physical Disk
Get-Disk

# Using Rufus or BalenaEtcher:
# 1. Select 'catalyst-kernel.img'
# 2. Target your USB Flash Drive
# 3. Write in 'DD Image mode' / Raw Write
```

### On Linux / macOS:
```bash
# Verify USB device node (e.g. /dev/sdb or /dev/rdisk2)
sudo dd if=catalyst-kernel.img of=/dev/sdX bs=4M status=progress conv=fsync
```

---

## 🖥️ 3. Physical Machine BIOS/Firmware Configuration
1. **Boot Mode:** Set to **Legacy BIOS / CSM Mode** (UEFI with CSM enabled).
2. **Secure Boot:** Set to **Disabled**.
3. **SATA Controller:** Set to **AHCI / IDE Mode**.
4. **Boot Priority:** Move USB Flash Drive to Top Priority.

---

## 🧪 4. Step-by-Step Hardware Acceptance Checklist

```
[ ] Step 1: Power ON -> BIOS POST -> Catalyst Bootloader banner appears.
[ ] Step 2: Microkernel initializes GDT, IDT, Frame Allocator, and Heap.
[ ] Step 3: ACPI dynamically discovers FADT / Power Control Ports.
[ ] Step 4: PCI Bus enumerates display, network, storage controllers.
[ ] Step 5: Screen switches from text mode to Catalyst Desktop (1280x768).
[ ] Step 6: Move physical mouse -> Pointer tracks smoothly across screen.
[ ] Step 7: Click Terminal window -> Type 'whoami' -> 'root' returned.
[ ] Step 8: Create file 'write /user/hello.txt Catalyst_Lives' -> Verified.
[ ] Step 9: Press Power Button or type 'shutdown' -> ACPI S5 Soft-Off cuts power.
[ ] Step 10: Power ON again -> Verify /user/hello.txt still exists on CPFS root.
```

---

## 🏆 Definition of Phase H Success:
> **"CatalystOS boots on bare metal, interacts with physical human input, persists file modifications across hardware reboots, and recovers gracefully from power loss without emulator intervention."**
