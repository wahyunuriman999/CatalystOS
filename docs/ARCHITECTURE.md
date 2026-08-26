# CATALYSTOS — SYSTEM ARCHITECTURE SPECIFICATION

**Version:** 0.2.0-dp1 (Developer Preview 1)
**Copyright:** (c) 2024-2026 Wahyu Nur Iman. All rights reserved.

---

## 1. Architectural Philosophy
CatalystOS is designed from first principles as a high-performance, capability-based modular microkernel. The design enforces strict mechanism/policy separation, minimum Trusted Computing Base (TCB), and hardware-enforced privilege isolation between kernel space (Ring 0) and userspace (Ring 3).

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

## 2. Kernel Invariants
1. **Ring 0 / Ring 3 Privilege Separation:** No user code ever executes in supervisor mode. Transition is mediated strictly via `syscall`/`sysretq` with per-thread kernel stacks (`TSS.RSP0`).
2. **Per-Process Address Spaces:** Each process owns a unique PML4 root. Kernel mappings are shared exclusively in the upper half ($0xFFFF\_8000\_0000\_0000..0xFFFF\_FFFF\_FFFF\_FFFF$) with supervisor-only access permissions.
3. **$W\oplus X$ Memory Protection:** No mapped user page may have both `WRITABLE` and `EXECUTABLE` flags set simultaneously.
4. **Opaque Capabilities:** Userspace holds only opaque `CapabilityHandle` tokens. Endpoint access rights (`SEND`, `RECEIVE`, `CALL`) are validated dynamically in kernel memory.
5. **Bounded Resource Allocation:** All queues (IPC max 64 messages), message sizes (max 256 bytes), and table sizes are hard-bounded to eliminate resource exhaustion attack surfaces.
6. **Deterministic Failure & Rollback:** System updates use A/B slot coordination (`SlotA`/`SlotB`) with automatic boot-attempt recovery rollbacks.
