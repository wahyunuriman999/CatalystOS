<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# CATALYST OS ARCHITECTURAL INVARIANTS v0.2

This document defines the non-negotiable architectural rules that **ALL** Catalyst OS subsystems, kernel modules, and user-space servers must obey. These invariants are the foundation of the OS's stability, security, and performance.

## 1. Catalyst is NOT built on top of another OS
- **Invariant:** Catalyst OS must boot directly on bare metal (or hypervisor) and interact with hardware via its own drivers and kernel.
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
- **Violation:** Integrating POSIX signal handling directly into the native Rust kernel scheduler.

## 7. No data sent to the cloud without explicit consent
- **Invariant:** Catalyst OS does not transmit telemetry, analytics, crash dumps, or user data to external servers without undeniable, opt-in consent from the user.
- **Why:** Privacy is a fundamental human right. The user owns their device and data.
- **Violation:** A background telemetry daemon phoning home by default, or an AI agent silently uploading context to a cloud provider.

## 8. Memory safety in the core is mandatory
- **Invariant:** The kernel, system servers, and core drivers must be written in memory-safe languages (Rust). `unsafe` blocks must be minimized, rigorously audited, and documented.
- **Why:** To eliminate the majority of critical security vulnerabilities (buffer overflows, use-after-free).
- **Violation:** Writing a new core system component in C/C++, or using `unsafe` in Rust without justification and safety comments.

## 9. Synchronous IPC is forbidden for potentially blocking operations
- **Invariant:** IPC calls that may block indefinitely must be asynchronous.
- **Why:** To prevent priority inversion and system deadlocks caused by unresponsive user-space servers.
- **Violation:** The Window Server blocking synchronously on an unresponsive filesystem driver.

## 10. The Kernel must remain minimal
- **Invariant:** The Catalyst Kernel implements the minimum set of mechanisms that require hardware privilege: CPU management, memory management, scheduling, IPC, capability enforcement, and minimal hardware mediation. All policy and complex services run in user space.
- **Why:** To limit the blast radius of crashes and keep the most privileged codebase small and auditable, regardless of the final kernel architectural paradigm.
- **Violation:** Moving a network stack, complex filesystem, or full graphics driver into kernel space for "performance reasons".

## 11. Catalyst Core Purity
- **Invariant:** Compatibility requirements must never redefine the fundamental architecture of Catalyst Kernel. Windows/Linux/Android compatibility must remain primarily above the Catalyst kernel. Kernel extensions for compatibility are permitted ONLY when: (a) technically unavoidable, (b) minimal, (c) generic, (d) security-reviewed, (e) useful beyond one compatibility target.
- **Why:** Legacy operating system design must not dictate the design of a modern, clean-slate system.
- **Violation:** Adding bespoke Windows NT emulation syscalls directly into the core kernel when they could be modeled via user-space IPC.

## 12. GPU Minimum Privilege
- **Invariant:** GPU driver architecture must minimize the trusted computing base. Only operations requiring hardware privilege (modesetting, VRAM page table management, interrupt handling) may run in kernel space. Command submission, shader management, and rendering pipeline run in user space.
- **Why:** GPU stacks are massive and complex; putting them entirely in Ring 0 is a massive security and stability risk.
- **Violation:** Running the entire DirectX/Vulkan driver equivalent inside the kernel.

## 13. Fail-Secure, not Fail-Open
- **Invariant:** When a component fails, crashes, or encounters an unexpected state, it must default to a secure, restrictive state rather than granting access.
- **Why:** To prevent attackers from forcing errors to bypass security checks.
- **Violation:** A permission server returning `ALLOW` when its backing database is corrupted.

## 14. Explicit Resource Limits for all processes
- **Invariant:** Every process must be launched with explicit bounds on CPU, memory, and file handles.
- **Why:** To guarantee system stability and prevent rogue processes from starving the system.
- **Violation:** Allowing an application to allocate memory unbounded until the system invokes the OOM killer.

## 15. UI must remain responsive regardless of system load
- **Invariant:** The window manager and input compositor must run with real-time priority and cannot be starved by user applications or system background tasks.
- **Why:** To provide a fluid, premium user experience. A system that stutters feels broken.
- **Violation:** A heavy compilation task causing the mouse cursor to lag.

## 16. Deterministic Initialization
- **Invariant:** The boot sequence and system initialization must be deterministic, meaning services start in a well-defined, reproducible order based on explicit dependencies.
- **Why:** To ensure predictable boot times and eliminate race conditions during startup.
- **Violation:** Relying on arbitrary sleep timers to wait for a service to become available before starting another.

## 17. Standardized Asynchronous I/O
- **Invariant:** All system I/O (files, networks, devices) must utilize a unified, asynchronous completion-based model (similar to io_uring).
- **Why:** For maximum throughput and minimum latency without thread-per-connection overhead.
- **Violation:** Implementing a custom, blocking I/O path for a specific driver outside the standard async framework.

## 18. IPC Over Shared Memory for Bulk Data
- **Invariant:** Whenever bulk data (like video frames, large files) is transferred between domains, it must be done via capability-secured shared memory mapped into both address spaces, not by copying data across the IPC boundary.
- **Why:** To maintain zero-copy efficiency.
- **Violation:** Passing a 4MB texture buffer directly as the payload of an IPC message.

## 19. Performance Isolation
- **Invariant:** A compatibility runtime, background service, AI subsystem, or third-party application must not be allowed to degrade the latency-critical behavior of unrelated workloads beyond defined performance budgets. Each workload class operates within resource bounds enforced by the scheduler and memory manager.
- **Why:** Catalyst's core value proposition is "smooth under any load." If an AI indexer bug causes GPU frame pacing to stutter, or a Windows Runtime crash freezes the compositor, the OS has failed its fundamental promise.
- **Violation:** An AI semantic indexer consuming 100% CPU and starving a running game's frame thread. A Docker container exhausting memory and triggering OOM kills against the desktop compositor. A background compilation task causing mouse cursor lag.

## 20. Failure Containment
- **Invariant:** Failure of any user-space component must not require a kernel restart or compromise unrelated system components. Each service must be independently restartable without affecting the stability of the rest of the system.
- **Why:** This is the fundamental justification for choosing a modular microkernel architecture. If a crash in Spotify requires rebooting, or a Windows Runtime failure panics the kernel, the architecture has failed.
- **Violation:** A GPU user-space service crash causing a kernel panic. A network driver failure freezing the entire desktop. A filesystem service bug requiring full system reboot instead of service restart.
