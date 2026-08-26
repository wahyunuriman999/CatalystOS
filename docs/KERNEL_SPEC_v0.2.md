<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS Kernel Architecture Specification v0.2

## 1. Executive Summary

This document re-evaluates the core kernel and security architecture for Catalyst OS. Following the ARB's rejection of a premature lock on a hybrid microkernel, this v0.2 specification performs a fresh, matrix-driven evaluation of independent architectural decisions, leading to a definitive architecture that respects the newly established invariants.

## 2. Decision A: Kernel Architecture Evaluation

### 2.1 Evaluated Alternatives

*   **A1: Modular monolithic** (Linux-inspired, Rust-native): All core services in kernel space, extensible via loadable modules.
*   **A2: Pure microkernel** (L4/seL4-inspired): Minimal kernel (IPC, scheduling, VM). All other services in user space.
*   **A3: Hybrid kernel** (NT-inspired): Microkernel-like structure but running critical services (e.g., graphics, VFS) in Ring 0 for performance.
*   **A4: Capability-oriented kernel** (Zircon-inspired): Built entirely around capabilities, balancing microkernel isolation with pragmatic performance paths.
*   **A5: Modular microkernel** (custom Catalyst design): A minimal core that allows highly trusted modules to be dynamically linked into the kernel address space if required by strict latency thresholds, but isolated by default.
*   **A6: Exokernel** (Barrelfish-inspired): Extreme hardware multiplexing; all OS abstractions in user-space libOS.

### 2.2 Weighted Scoring Matrix (0-10 points per criterion)

| Criterion (Weight) | A1 (Mono) | A2 (Micro) | A3 (Hybrid) | A4 (Cap) | A5 (Cat Mod-Micro) | A6 (Exo) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| Gaming latency (15%) | 9 (1.35) | 3 (0.45) | 8 (1.20) | 7 (1.05) | 9 (1.35) | 10 (1.50) |
| IPC overhead (12%) | 9 (1.08) | 4 (0.48) | 7 (0.84) | 6 (0.72) | 8 (0.96) | 10 (1.20) |
| Memory footprint (10%) | 5 (0.50) | 9 (0.90) | 6 (0.60) | 8 (0.80) | 8 (0.80) | 9 (0.90) |
| Driver isolation (10%) | 2 (0.20) | 10 (1.00) | 6 (0.60) | 9 (0.90) | 9 (0.90) | 10 (1.00) |
| GPU arch fit (10%) | 8 (0.80) | 4 (0.40) | 9 (0.90) | 7 (0.70) | 9 (0.90) | 8 (0.80) |
| Security (10%) | 3 (0.30) | 10 (1.00) | 5 (0.50) | 10 (1.00) | 9 (0.90) | 8 (0.80) |
| Fault isolation (8%) | 2 (0.16) | 10 (0.80) | 5 (0.40) | 9 (0.72) | 9 (0.72) | 9 (0.72) |
| Compat feasibility (8%) | 9 (0.72) | 5 (0.40) | 8 (0.64) | 7 (0.56) | 8 (0.64) | 4 (0.32) |
| Low-end HW (5%) | 7 (0.35) | 6 (0.30) | 6 (0.30) | 7 (0.35) | 8 (0.40) | 8 (0.40) |
| Dev complexity (5%) | 8 (0.40) | 4 (0.20) | 5 (0.25) | 4 (0.20) | 6 (0.30) | 2 (0.10) |
| Debugging (4%) | 4 (0.16) | 8 (0.32) | 5 (0.20) | 7 (0.28) | 7 (0.28) | 3 (0.12) |
| Maintainability (3%) | 3 (0.09) | 9 (0.27) | 5 (0.15) | 8 (0.24) | 8 (0.24) | 4 (0.12) |
| **TOTAL (100%)** | **6.11** | **6.52** | **6.58** | **7.52** | **8.39** | **7.98** |

### 2.3 Recommendation: A5 (Modular Microkernel)

*   **Requirements Addressed:** Combines strict isolation defaults (microkernel) with the ability to dynamically host critical paths (like minimum-privilege GPU modesetting) in ring 0 for latency (hybrid/modular).
*   **Alternatives:** A4 is excellent but rigid; A3 risks bloat; A1 violates isolation invariants.
*   **Trade-offs:** High engineering cost to develop the secure module boundary.
*   **Selected Decision:** A5 Modular microkernel.
*   **Rejected Alternatives:** A1 (fails security/isolation), A2 (fails gaming latency/IPC overhead), A3 (security compromises).
*   **Consequences:** Requires custom Rust-based linker/loader for kernel modules.
*   **Reversibility:** Low. The fundamental IPC and memory boundaries will be deeply entrenched.

## 3. Decision B: Security Architecture

### 3.1 Evaluated Alternatives

*   **B1: Capability-based security** (seL4-style): Object references convey unforgeable authority.
*   **B2: ACL/DAC + MAC** (Linux-style): Identities and central policy.
*   **B3: Hybrid capability + role-based**: Capabilities for fast-path IPC, roles for high-level user administration.
*   **B4: Object-oriented security**: Language-level typing enforces security (requires radical hardware/language co-design).

### 3.2 Recommendation: B1 (Capability-based)

*   **Requirements Addressed:** Zero-trust, decentralized policy, robust IPC security.
*   **Alternatives:** B2 is too centralized and prone to ambient authority issues. B3 adds unnecessary complexity.
*   **Selected Decision:** B1 Pure Capability-based security.
*   **Rejected Alternatives:** B2 (fails principle of least privilege easily), B4 (too experimental).
*   **Consequences:** All IPC requires passing capability tokens. No ambient global state.
*   **Reversibility:** Very Low. Dictates the syscall API.

## 4. GPU Architecture (Minimum-Privileged)

Following the "Minimum TCB + minimum privileged GPU component" invariant:

*   **Kernel (Minimum TCB):** Modesetting (display output timing), VRAM page table root updates, interrupt acknowledgment/masking, and critical thermal/power thresholds.
*   **User-Space:** Command submission queues, shader compilation/management, rendering pipelines, Vulkan ICD implementation, memory allocation strategy.
*   **Command Submission Path:** User-space rendering clients build command buffers locally. They map a shared memory ring buffer with the user-space GPU Scheduler. The Scheduler constructs final hardware queues and executes a single lightweight syscall (or uses memory-mapped doorbell registers) to notify the kernel, which performs a rapid capability check and rings the hardware doorbell.
*   **Crash Recovery:** If a user-space Vulkan ICD crashes, the rendering client dies, but the display remains active. If the user-space GPU Scheduler crashes, the kernel resets the GPU hardware state, spawns a new Scheduler, and compositor clients transparently reconnect. The system does not panic.
*   **Security Boundaries:** Total isolation between rendering clients. VRAM is mapped per-process by the kernel based on capabilities issued by the user-space Scheduler.
*   **Latency Analysis:** Direct memory-mapped doorbell registers avoid syscall overhead for command submission on supported hardware, achieving zero-syscall render loops.

## 5. Core Subsystem Design

### 5.1 Syscall Architecture
- **Design:** Capability-centric `sys_invoke`. No traditional POSIX syscalls. Everything is an invocation on a capability reference.

### 5.2 Process Model
- A "Process" is strictly a VSpace (memory map) and a CSpace (capability map). No `fork()`. Process creation is explicit assembly.

### 5.3 Thread Model
- 1:1 kernel threads scheduled across CPU cores. Threads hold execution state, Processes hold resources.

### 5.4 Context Switching
- Tickless (APIC timer), lazy FPU saving (only saves AVX/SSE when the new thread attempts to use them).

### 5.5 IPC Architecture (Fast-Path + Standard)

IPC has two paths to satisfy both security and latency requirements:

**Standard IPC (Control Path):**
- Register-based synchronous messages for small control data (<64 bytes).
- Kernel mediates, validates capabilities, copies registers.
- Used for: service discovery, permission requests, configuration.

**Fast-Path IPC (Data Path):**
- Capability-secured shared memory ring buffers mapped into both address spaces.
- Zero-copy: producer writes to shared buffer, consumer reads directly.
- Kernel role: initial capability grant and revocation only — not in the data path.
- Synchronization: lightweight futex-like mechanism or memory-mapped doorbell.
- Used for: GPU command submission, audio streaming, display compositing, file I/O bulk transfer.

```
Normal IPC:     App → Kernel → Service     (safe, mediated)
Fast-Path IPC:  App ──────────→ Service    (shared memory, capability-gated)
                     zero-copy
```

**Design rationale:** A pure-microkernel IPC-only approach would impose copy overhead on every graphics frame and audio buffer. Fast-path shared memory preserves microkernel security (kernel controls capability grants) while achieving monolithic-kernel data throughput.

### 5.6 Workload-Aware Scheduler

The scheduler classifies threads into workload classes and optimizes per-class:

| Class | Priority | Optimization Target | Example |
|:---|:---|:---|:---|
| **System Critical** | Highest | Guaranteed execution, never starved | Kernel threads, capability server |
| **Interactive** | High | Input-to-display latency <16ms | Window compositor, input handler |
| **Gaming** | High | Frame pacing, CPU-GPU sync, minimum jitter | Game main thread, render thread |
| **Developer** | Medium-High | Throughput, parallel compilation | Compiler, linker, test runner |
| **Normal** | Medium | Fair time-sharing | Office apps, browser, file manager |
| **Background** | Low | Minimal CPU/power, throttled under pressure | AI indexer, cloud sync, updates |
| **Idle** | Lowest | Zero impact when other classes need resources | Telemetry (opt-in), pre-caching |

**Workload detection:** Threads are classified by their creator's manifest declaration + runtime heuristics (e.g., a thread calling GPU submit APIs repeatedly is auto-promoted to Gaming class).

**Power awareness:** On battery, Background and Idle classes are aggressively throttled. Gaming class adjusts thermal/power budgets dynamically.

**Performance Isolation (Invariant #19):** Each class has resource budgets. A Background process cannot starve a Gaming thread regardless of CPU demand.

### 5.7 Interrupt Handling
- Top-half in kernel (ACK, mask, queue event). Bottom-half in user-space threads waiting on the interrupt capability.

### 5.8 Driver Model
- Default to user-space. Kernel space strictly reserved for core system routing (IOMMU, minimal PCIe, APIC) and minimum-TCB GPU requirements.

## 6. GPU Architecture (Minimum-Privileged, Dual-Path)

Following the "Minimum TCB + minimum privileged GPU component" invariant:

**Kernel (Minimum TCB):** Modesetting (display output timing), VRAM page table root updates, interrupt acknowledgment/masking, critical thermal/power thresholds.

**User-Space:** Command submission queues, shader compilation/management, rendering pipelines, Vulkan ICD implementation, memory allocation strategy.

**Dual-Path Command Submission:**

```
             Application
                  │
                  ↓
          Catalyst Graphics API
                  │
          ┌───────┴────────┐
          ↓                ↓
       Fast Path        Safe Path
    (shared queues)      (IPC)
    (doorbell regs)     (kernel-mediated)
          │                │
          └───────┬────────┘
                  ↓
           GPU Subsystem
                  ↓
              Hardware
```

- **Fast Path:** Memory-mapped doorbell registers (where hardware supports it). User-space builds command buffer → writes to shared ring → rings doorbell. Kernel not in critical path. Maximum performance.
- **Safe Path (Fallback):** Standard IPC-based submission through kernel. Used when hardware lacks doorbell support, or during GPU recovery after crash. Always available.
- **Crash Recovery:** User-space Vulkan ICD crash → rendering client dies, display stays active. GPU Scheduler crash → kernel resets GPU state, spawns new Scheduler, compositor clients reconnect. System never panics from GPU failure (Invariant #20).
- **Security:** VRAM mapped per-process by kernel based on capabilities. Total isolation between rendering clients.

## 7. Catalyst Runtime Architecture (First-Class Subsystem)

Compatibility runtimes are a first-class architectural subsystem, not an afterthought:

```
             Catalyst Core
                  │
    ┌─────────────┼─────────────┐
    ↓             ↓             ↓
 Catalyst      Windows       Linux        Android
  Native       Runtime       Runtime       Runtime
 (Tier 1)     (Tier 3)     (Tier 2)      (Tier 4)
    │             │             │             │
    └─────────────┼─────────────┼─────────────┘
                  ↓
            Catalyst APIs
                  ↓
            Catalyst Kernel
```

**Compatibility Tiering (ARB-approved):**

| Tier | Ecosystem | Priority | Status |
|:---|:---|:---|:---|
| 1 | Catalyst Native | Highest | Primary development target |
| 2 | Linux | High | ELF/syscall translation — large developer ecosystem |
| 3 | Windows | High | Win32 API translation — Office, Steam, games |
| 4 | Android | Medium | APK/ART runtime — mobile app ecosystem |
| 5 | macOS | Low | Portable standards only — NOT a near-term goal |

**Core Purity enforcement:** Each runtime is sandboxed in user-space. Kernel changes for compatibility require passing the 5-gate test (Invariant #11).

**Anti-cheat security boundary:** Catalyst does NOT compromise its security architecture for kernel-level anti-cheat. The approach is:
1. Catalyst provides a Game Security API with attestation capabilities.
2. Anti-cheat vendors can use this API within Catalyst's security model.
3. If an anti-cheat requires kernel modifications incompatible with Catalyst security, it is declared **unsupported** rather than breaking the OS.
4. Order: Catalyst Security → Catalyst Game Security API → Anti-cheat compatibility. Never: Anti-cheat → modify kernel.

## 8. Architectural Provisionality

**A5 Modular Microkernel is the provisional kernel architecture (v0.2).**

This is the best current hypothesis based on weighted evaluation, NOT a mathematically proven optimal architecture. The score of 8.39/10 is a decision aid, not proof.

Validation requires real benchmarks:
- IPC latency measurement (M5/M6)
- Scheduler fairness testing (M4)
- GPU submission latency (M10)
- Memory footprint measurement (M2)

If benchmark evidence demonstrates that A5 fundamentally cannot meet performance targets, the architecture may be revised with a formal ADR and ARB approval.

