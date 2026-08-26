<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# CATALYST OS ARCHITECTURAL INVARIANTS v0.1

This document defines the non-negotiable architectural rules that **ALL** Catalyst OS subsystems, kernel modules, and user-space servers must obey. These invariants are the foundation of the OS's stability, security, and performance.

## 1. Catalyst is NOT built on top of another OS
- **Invariant:** Catalyst OS must boot directly on bare metal (or hypervisor) and interact with hardware via its own drivers and microkernel.
- **Why:** To guarantee full control over the entire stack, performance predictability, and security boundaries. Wrapping another kernel (like Linux) introduces foreign security assumptions and legacy bloat.
- **Violation:** Depending on Linux syscalls, using a POSIX compatibility layer in the core kernel, or relying on another OS for bootstrap.

## 2. Every resident service must justify its existence
- **Invariant:** Any background service or daemon must have a rigorously justified and minimal footprint (CPU, RAM, I/O, wakeups). Idle services must consume zero CPU cycles.
- **Why:** To maximize battery life and system responsiveness. "Bloatware" kills performance.
- **Violation:** A service polling continuously, consuming memory when unused, or starting automatically without a clear, immediate user benefit.

## 3. Security boundaries cannot be weakened for convenience
- **Invariant:** The capability-based security model and process isolation must be strictly enforced, even if it makes application development or debugging harder.
- **Why:** Convenience compromises inevitably become exploit vectors. Security is binary.
- **Violation:** Introducing a "global root" override, granting ambient access to the filesystem to bypass capability checks, or disabling ASLR for a specific app.

## 4. Updates must be atomic, verifiable, and rollback-capable
- **Invariant:** System updates are applied atomically to a passive partition. They must be cryptographically verified before booting, and the system must automatically roll back on boot failure.
- **Why:** To prevent bricked devices and ensure a reliable, stress-free update experience for the user.
- **Violation:** Modifying live system files in-place during an update, failing to verify signatures, or requiring manual user intervention to recover from a bad update.

## 5. User data is always separate from system images
- **Invariant:** System partitions are immutable and separated from the user data partition. The OS can be completely wiped and reinstalled without touching user documents.
- **Why:** For system integrity, easy factory resets, and protecting user data during OS upgrades or corruption.
- **Violation:** Storing system configuration files in the user's home directory or user data in `/system`.

## 6. Compatibility runtimes are separate from the core OS
- **Invariant:** Subsystems running foreign binaries (e.g., Linux or Windows compatibility layers) must run isolated in user-space, communicating via standard IPC, and cannot pollute the core system architecture.
- **Why:** To maintain a clean, secure native environment while providing necessary compatibility without compromising the host OS.
- **Violation:** Integrating POSIX signal handling directly into the native Rust microkernel scheduler.

## 7. No data sent to the cloud without explicit consent
- **Invariant:** Catalyst OS does not transmit telemetry, analytics, crash dumps, or user data to external servers without undeniable, opt-in consent from the user.
- **Why:** Privacy is a fundamental human right. The user owns their device and data.
- **Violation:** A background telemetry daemon phoning home by default, or an AI agent silently uploading context to a cloud provider.

## 8. Memory safety in the core is mandatory
- **Invariant:** The microkernel, system servers, and core drivers must be written in memory-safe languages (Rust). `unsafe` blocks must be minimized, rigorously audited, and documented.
- **Why:** To eliminate the majority of critical security vulnerabilities (buffer overflows, use-after-free).
- **Violation:** Writing a new core system component in C/C++, or using `unsafe` in Rust without justification and safety comments.

## 9. Synchronous IPC is forbidden for potentially blocking operations
- **Invariant:** IPC calls that may block indefinitely must be asynchronous.
- **Why:** To prevent priority inversion and system deadlocks caused by unresponsive user-space servers.
- **Violation:** The Window Server blocking synchronously on an unresponsive filesystem driver.

## 10. The Kernel must remain minimal (Microkernel Principle)
- **Invariant:** The kernel is responsible ONLY for memory management, scheduling, and IPC. Drivers, filesystems, and networking run in user-space.
- **Why:** To limit the blast radius of crashes and keep the most privileged codebase small and auditable.
- **Violation:** Moving a network stack or graphics driver into kernel space for "performance reasons".

## 11. Fail-Secure, not Fail-Open
- **Invariant:** When a component fails, crashes, or encounters an unexpected state, it must default to a secure, restrictive state rather than granting access.
- **Why:** To prevent attackers from forcing errors to bypass security checks.
- **Violation:** A permission server returning `ALLOW` when its backing database is corrupted.

## 12. Explicit Resource Limits for all processes
- **Invariant:** Every process must be launched with explicit bounds on CPU, memory, and file handles.
- **Why:** To guarantee system stability and prevent rogue processes from starving the system.
- **Violation:** Allowing an application to allocate memory unbounded until the system invokes the OOM killer.

## 13. UI must remain responsive regardless of system load
- **Invariant:** The window manager and input compositor must run with real-time priority and cannot be starved by user applications or system background tasks.
- **Why:** To provide a fluid, premium user experience. A system that stutters feels broken.
- **Violation:** A heavy compilation task causing the mouse cursor to lag.

## 14. Deterministic Initialization
- **Invariant:** The boot sequence and system initialization must be deterministic, meaning services start in a well-defined, reproducible order based on explicit dependencies.
- **Why:** To ensure predictable boot times and eliminate race conditions during startup.
- **Violation:** Relying on arbitrary sleep timers to wait for a service to become available before starting another.

## 15. Standardized Asynchronous I/O
- **Invariant:** All system I/O (files, networks, devices) must utilize a unified, asynchronous completion-based model (similar to io_uring).
- **Why:** For maximum throughput and minimum latency without thread-per-connection overhead.
- **Violation:** Implementing a custom, blocking I/O path for a specific driver outside the standard async framework.
