<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS Kernel Architecture Specification v0.1

## 1. Executive Summary

This document specifies the core architecture for Catalyst OS, a general-purpose PC operating system written in Rust targeting x86-64 (extensible to ARM64). The primary objective is to define a kernel architecture that balances gaming performance, system security, driver isolation, and hardware efficiency.

## 2. Architecture Evaluation

### 2.1 Architectures Evaluated

1. **Monolithic Kernel (e.g., Linux, BSD)**
   - All core services (VFS, IPC, drivers, networking) run in kernel space.
2. **Microkernel (e.g., L4, seL4, Mach)**
   - Only minimal mechanisms (IPC, virtual memory, thread scheduling) run in kernel space; everything else runs in user space.
3. **Hybrid Kernel (e.g., Windows NT, XNU, Haiku)**
   - Core services in kernel space for performance, but heavily modularized, with some services and drivers in user space.
4. **Modular Kernel (e.g., Solaris, Linux with LKMs)**
   - Monolithic base but heavily reliant on loadable modules at runtime.
5. **Capability-Oriented Kernel (e.g., seL4, Fuchsia/Zircon)**
   - Access to all resources (memory, IPC, objects) is governed by unforgeable capabilities. Can be combined with micro or hybrid approaches.
6. **Exokernel-Inspired (e.g., Nemesis, Barrelfish)**
   - Kernel provides extreme hardware multiplexing with almost no abstractions; libraries (libOS) provide abstractions in user space.

### 2.2 Evaluation Criteria & Analysis

| Criterion | Monolithic | Microkernel | Hybrid | Modular | Capability-Oriented | Exokernel |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Latency (Syscall/IPC/Int)** | Excellent (direct calls) | Poor (many context switches) | Good (direct for critical paths) | Excellent | Good to Poor (depends on size) | Excellent (libOS bypass) |
| **IPC Overhead** | Minimal | High (message passing) | Moderate | Minimal | Moderate to High | Low |
| **Driver Isolation** | Poor (kernel panics) | Excellent (user-space faults) | Good (user-mode driver frameworks) | Poor | Excellent (capability bounds) | Excellent |
| **Gaming Performance** | Excellent | Poor (IPC overhead) | Excellent (kernel graphics) | Excellent | Good | Excellent |
| **Low-end HW Efficiency** | Good | Moderate (memory overhead) | Good | Good | Moderate | Excellent |
| **Debugging Complexity** | High (kernel state) | Low (user-space services) | Moderate | High | Moderate | High |
| **Security/Privilege** | Poor (ring 0 monolithic) | Excellent (TCB is small) | Moderate | Poor | Excellent | Good |
| **Maintainability** | Poor (spaghetti potential) | Excellent (decoupled) | Moderate (clear subsystems) | Poor | Good | Moderate |
| **ABI Compatibility** | Excellent (in-kernel) | Moderate (user-space server) | Excellent (kernel subsystems) | Excellent | Moderate | Good |
| **Memory Footprint** | Large | Small (kernel) + Large (IPC) | Moderate | Large | Moderate | Small |
| **Developer Complexity** | High | Low (services) / High (IPC) | Moderate | High | High (learning curve) | Extreme |
| **Portability** | Good | Excellent | Good | Good | Good | Poor (tied to HW) |

### 2.3 Evidence-Based Recommendation

**Decision:** We adopt a **Hybrid Microkernel Architecture with Capability-Based Security**.

- **Requirements addressed:** Needs high gaming performance (low latency, fast GPU access) but strong driver isolation and modern security.
- **Alternatives considered:** Pure monolithic provides performance but fails driver isolation. Pure microkernel provides isolation but fails gaming performance due to IPC overhead in critical graphics/audio paths.
- **Trade-offs:** Implementing a hybrid requires careful delineation of which subsystems belong in ring 0 vs ring 3. The capability model adds complexity to the API but secures the hybrid nature.
- **Rationale:** By keeping the core kernel minimal (like a microkernel) but allowing performance-critical subsystems (like the display compositor and core graphics drivers) to run in kernel-space or fast-path shared memory environments, we achieve NT-like performance with L4-like security. The use of Rust allows safe in-kernel components without the memory safety bugs of C.
- **Measurable consequences:** Syscall latency will be slightly higher than monolithic for IPC-based services, but graphics/input latency will be identical. A crash in a network or USB driver will not panic the kernel.

## 3. Core Subsystem Design

### 3.1 Syscall Architecture
- **Design:** `syscall` / `sysret` on x86-64 using a capability-based invocation model.
- **Details:** Syscalls do not map to static OS functions but invoke capabilities (e.g., `sys_invoke(capability_id, operation, args)`).
- **Rationale:** Reduces the syscall surface area and intrinsically enforces access control at the boundary.

### 3.2 Process & Thread Model
- **Process Model:** Processes are simply virtual address spaces (VSpaces) and a capability space (CSurc). No inherent UNIX-like `fork()`. Processes are created by spawning an empty VSpace and mapping executable memory.
- **Thread Model:** 1:1 kernel-level threading. Threads are the executable entities scheduled by the kernel.
- **Rationale:** Separating resource ownership (Process) from execution (Thread) simplifies scheduling and capability management.

### 3.3 Context Switching Strategy
- **Design:** Preemptive, tickless scheduler using the LAPIC timer.
- **Details:** Context switches save general-purpose registers and use `xsave`/`xrstor` for AVX/SSE state. Floating-point state is lazily saved only when used by the new thread to minimize switch time.
- **Rationale:** Lazy FPU saving is crucial for fast context switching in I/O heavy workloads, while gaming threads get dedicated cores with minimal preemption.

### 3.4 IPC Mechanism
- **Design:** Synchronous message passing for control, asynchronous shared memory (Ring Buffers) for data.
- **Details:** Core IPC uses registers for small messages (up to 64 bytes). Large messages require establishing a shared memory mapping governed by a capability.
- **Rationale:** Register-only IPC is ultra-fast for microkernel server coordination. Shared memory prevents copying overhead for graphics and disk I/O, matching monolithic performance.

### 3.5 Interrupt Handling Architecture
- **Design:** Top-half / Bottom-half split. Top-half in kernel (minimal, acknowledges IRQ, masks it, and signals an event).
- **Details:** The event wakes up a waiting user-space driver thread (bottom-half).
- **Rationale:** Keeps interrupt latency extremely low. Complex driver logic is moved to user-space where it can be preempted and isolated.

### 3.6 Driver Model
- **Design:** User-Space Driver Framework (UDF) for most devices; In-Kernel for core system (timer, interrupt controller, IOMMU, basic PCIe).
- **Details:** Network, USB, Audio, and Storage drivers run as isolated user-space processes. GPU drivers are split: DRM/KMS-equivalent runs in kernel for modesetting and VRAM management, while 3D rendering (Vulkan/DirectX translation) is entirely user-space.
- **Rationale:** Maximizes stability. A crashing WiFi driver simply restarts. The split GPU model is necessary because full user-space GPU drivers lack the privileged access needed for rapid VRAM management and display output scheduling required for gaming.
