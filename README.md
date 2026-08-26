<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS

> **Efficient by default. Responsive by design. Power when you need it.**

Catalyst OS is a modern, general-purpose PC operating system built from first principles in Rust, structured as a high-performance capability-based modular microkernel.

---

## 🏛️ System Architecture

```
+-------------------------------------------------------------------------+
|                              USERSPACE (Ring 3)                         |
|  +--------------+  +--------------+  +--------------+  +-------------+  |
|  | Applications |  | Desktop / UI |  | User Daemons |  | libcatalyst |  |
|  +-------+------+  +-------+------+  +-------+------+  +------+------+  |
|          |                 |                 |                |         |
|          +-----------------+-----------------+----------------+         |
|                                    | (IPC / Syscalls)                   |
+------------------------------------|------------------------------------+
|                                    v                                    |
|                              KERNEL (Ring 0)                            |
|  +-------------------------------------------------------------------+  |
|  | Syscall Dispatcher (MSR LSTAR) & Capability Table Enforcement     |  |
|  +-------------------------------------------------------------------+  |
|  | Microkernel Core:                                                 |  |
|  | - Preemptive Scheduler & Thread Context Switching                 |  |
|  | - Generational Endpoint IPC Registry (Bounded Queue & Wakeups)    |  |
|  | - 4-Level Paging Address Spaces & Frame Allocator                 |  |
|  | - VFS Subsystem (Inodes, File Descriptors, RamFS Root)            |  |
|  | - Service Manager, Security Quotas, W^X Enforcer, Watchdog       |  |
|  +-------------------------------------------------------------------+  |
|                                    |                                    |
|                                    v                                    |
|                                HARDWARE                                 |
|  +-------------------------------------------------------------------+  |
|  | CPU (x86_64), APIC/PIC, Serial UART, Framebuffer, PCI, VirtIO     |  |
|  +-------------------------------------------------------------------+  |
+-------------------------------------------------------------------------+
```

---

## 📦 Current Release Status

**Milestone:** `Developer Preview 1 (DP1)`

- **Privilege Separation:** Ring 0 (Kernel) & Ring 3 (User) with per-thread kernel stacks (`TSS.RSP0`).
- **Memory Management:** 4-level paging, bitmap frame allocation, and safe userspace memory copyin/copyout.
- **Multitasking:** Preemptive round-robin scheduler with timer interrupt preemption and thread context switching.
- **Capability IPC:** Generational endpoint validation, opaque capability handles, blocking receive, atomic wakeups, and synchronous RPC.
- **Storage & VFS:** Hierarchical Inodes/VNodes, RamFS root, File Descriptor tables, and CPKG package installer.
- **Reliability & Recovery:** A/B system slot updates with automatic 3-boot attempt rollback, and kernel watchdog monitoring.
- **Verification:** 33 discrete runtime verification tests in CI with zero faults.

---

## 🛠️ Building & Running

### Prerequisites
- Rust Nightly (`rustup default nightly`)
- Rust components: `rust-src`, `llvm-tools-preview`
- QEMU (`qemu-system-x86_64`)

### Deterministic Build
```bash
# On Linux / macOS / CI:
bash scripts/build.sh

# On Windows (PowerShell):
powershell -ExecutionPolicy Bypass -File scripts/build.ps1
```

### Running in QEMU
```bash
powershell -ExecutionPolicy Bypass -File tools/run-qemu.ps1
```

---

## 📚 Technical Documentation

| Document | Description |
| :--- | :--- |
| [Architecture Specification](docs/ARCHITECTURE.md) | System layers, boundaries, and microkernel invariants |
| [Syscall ABI Specification](docs/SYSCALL_ABI.md) | Calling conventions, syscall numbers, and memory safety contracts |
| [Productization Baseline](docs/PRODUCTIZATION_BASELINE.md) | Forensic audit of repository, subsystems, and security models |
| [Product Readiness Report](docs/PRODUCT_READINESS_REPORT.md) | Subsystem classification matrix, CI oracle, and roadmap to Beta |
| [Architectural Invariants](docs/ARCHITECTURAL_INVARIANTS_v0.2.md) | 20 non-negotiable architectural rules |
| [Kernel Specification](docs/KERNEL_SPEC_v0.2.md) | Complete kernel architectural evaluation |

---

## ⚖️ License

Proprietary and Confidential. Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved.
