<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Architecture Decision Matrix v0.1

This document formalizes the 10 major architectural decisions for Catalyst OS, scoring alternatives against the weighted criteria defined by the ARB.

*(Scoring criteria weights: Gaming latency 15%, IPC overhead 12%, Memory footprint 10%, Driver isolation 10%, GPU arch fit 10%, Security 10%, Fault isolation 8%, Compat feasibility 8%, Low-end HW 5%, Dev complexity 5%, Debugging 4%, Maintainability 3%)*

## 1. Kernel Architecture Type
*   **Requirements:** Low latency, strong isolation, scalable.
*   **Alternatives:** A1 (Monolithic), A2 (Microkernel), A5 (Modular Microkernel).
*   **Selected Decision:** A5 (Modular Microkernel).
*   **Trade-offs:** High upfront engineering cost vs. long-term stability and latency control.
*   **Rejected:** Monolithic (poor fault isolation), Pure Microkernel (poor IPC latency for graphics).
*   **Consequences:** Requires custom loading boundaries and strict kernel boundary definitions.
*   **Reversibility:** Low.

## 2. Security Model
*   **Requirements:** Decentralized, fail-secure, fine-grained.
*   **Alternatives:** Capability-based, ACL/MAC, Role-based.
*   **Selected Decision:** Capability-based (seL4-style).
*   **Trade-offs:** High developer learning curve vs. mathematically provable security boundaries.
*   **Rejected:** ACL/MAC (susceptible to ambient authority bypasses).
*   **Consequences:** All IPC and resource access must present capability tokens.
*   **Reversibility:** Very Low.

## 3. GPU Driver Boundary
*   **Requirements:** Zero-latency command submission, strict fault isolation.
*   **Alternatives:** Full monolithic (Linux DRM), Full user-space (unprivileged), Split minimum-privilege (Catalyst model).
*   **Selected Decision:** Split minimum-privilege.
*   **Trade-offs:** Complexity in splitting state management vs. system stability on GPU hangs.
*   **Rejected:** Monolithic (kernel panics on GPU driver bugs).
*   **Consequences:** Requires carefully designed user-space GPU scheduling daemons.
*   **Reversibility:** Moderate (driver models can be iterated).

## 4. IPC Mechanism
*   **Requirements:** Microsecond latency, zero-copy for bulk data.
*   **Alternatives:** Socket-based, Message queues with copy, Register-fastpath + Shared Memory.
*   **Selected Decision:** Register-fastpath + Shared Memory.
*   **Trade-offs:** Memory management complexity vs. throughput.
*   **Rejected:** Socket/Copy-based (fails gaming latency and IPC overhead requirements).
*   **Consequences:** Applications must be engineered to handle shared memory ring buffers.
*   **Reversibility:** Low.

## 5. Driver Model
*   **Requirements:** Fault isolation, rapid recovery, ease of debugging.
*   **Alternatives:** In-kernel, Hybrid (Windows UMDF), Pure User-Space (Catalyst UDF).
*   **Selected Decision:** Pure User-Space (with core exemptions for TCB).
*   **Trade-offs:** Slight interrupt latency increase vs. absolute system stability.
*   **Rejected:** In-kernel (frequent source of kernel panics).
*   **Consequences:** I/O heavy operations require efficient IPC.
*   **Reversibility:** High (drivers can be moved to/from kernel space during dev).

## 6. Filesystem Strategy
*   **Requirements:** Atomic updates, crash consistency, fast async I/O.
*   **Alternatives:** VFS in kernel (Ext4/ZFS), User-space VFS + micro-servers, Log-structured user-space.
*   **Selected Decision:** User-space VFS + micro-servers (io_uring style async).
*   **Trade-offs:** IPC overhead on tiny file reads vs. driver isolation.
*   **Rejected:** In-kernel VFS (bloats the kernel).
*   **Consequences:** All VFS operations are capability-mediated IPC calls.
*   **Reversibility:** Low.

## 7. Scheduler Design
*   **Requirements:** Real-time UI responsiveness, core pinning for games.
*   **Alternatives:** CFS (Linux style), O(1), Capability-based time-slice donations.
*   **Selected Decision:** Capability-based time-slice donations.
*   **Trade-offs:** Complex priority management vs. guaranteed real-time UI/Gaming performance without priority inversion.
*   **Rejected:** CFS (prone to starvation under heavy I/O loads).
*   **Consequences:** Schedulers must manage thread budgets explicitly.
*   **Reversibility:** Moderate.

## 8. Compatibility Layer Boundary
*   **Requirements:** Run foreign code without compromising native security or invariants.
*   **Alternatives:** In-kernel emulation (WSL1 style), Hypervisor/VM (WSL2 style), User-space library OS (Wine style).
*   **Selected Decision:** User-space library OS running inside an unprivileged sandbox.
*   **Trade-offs:** Emulation overhead vs. kernel purity.
*   **Rejected:** In-kernel emulation (violates kernel minimalism invariant).
*   **Consequences:** Native performance for Linux/Windows apps will rely heavily on efficient user-space IPC translation.
*   **Reversibility:** High.

## 9. Memory Management Approach
*   **Requirements:** Fast allocations, strict bounds, zero fragmentation.
*   **Alternatives:** Unified kernel pager, User-space pagers (Mach style), Capability-driven untyped memory (seL4).
*   **Selected Decision:** Capability-driven untyped memory.
*   **Trade-offs:** Application must manage its own page tables (via libOS) vs. ultimate security and explicit resource limits.
*   **Rejected:** Unified kernel pager (monolithic approach).
*   **Consequences:** Highly complex user-space runtime required to manage memory transparently to standard apps.
*   **Reversibility:** Low.

## 10. Package/Application Model
*   **Requirements:** Immutable system, atomic updates, distinct app boundaries.
*   **Alternatives:** Global filesystem (Debian/RPM), Containerized/Flatpak, Cryptographic App Images.
*   **Selected Decision:** Cryptographic App Images (isolated bundles).
*   **Trade-offs:** Disk space duplication vs. dependency hell elimination.
*   **Rejected:** Global filesystem (violates immutable system invariant).
*   **Consequences:** Every application runs in a strict VSpace/CSpace container.
*   **Reversibility:** Moderate.
