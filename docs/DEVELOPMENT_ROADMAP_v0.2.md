<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS: Development Roadmap v0.2

## Core Philosophy
Core OS correctness precedes all filesystem and compatibility work. Development progresses linearly through foundational components.

---

## M0: Architecture (Current)
- **Objective:** Finalize OS architecture, compatibility strategy, test plans, and filesystem spec.
- **Dependencies:** None.
- **Implementation Tasks:** ARB review of v0.2 specs.
- **Tests:** Architectural sign-off.
- **Acceptance Criteria:** Approval of M0 specifications.
- **Measurable Benchmark:** N/A.
- **Risks:** Changing requirements.
- **Complexity:** Low.

## M1: Boot (UEFI → kernel → console)
- **Objective:** Successfully hand off from UEFI to Rust kernel environment.
- **Dependencies:** M0.
- **Implementation Tasks:** UEFI bootloader, kernel ELF loading, long mode initialization, basic serial logging, GDT, IDT.
- **Tests:** Boot QEMU, catch a divide-by-zero exception.
- **Acceptance Criteria:** Kernel boots, prints to serial, handles CPU exceptions safely.
- **Measurable Benchmark:** Boot time (UEFI to `kmain`) < 50ms.
- **Risks:** Toolchain configuration issues, UEFI idiosyncrasies.
- **Complexity:** Low.

## M2: Memory (physical + virtual + heap)
- **Objective:** Establish safe, dynamic memory allocation.
- **Dependencies:** M1.
- **Implementation Tasks:** Physical frame allocator, page table management, kernel heap allocator.
- **Tests:** Allocate and free random blocks; stress test page mapping/unmapping.
- **Acceptance Criteria:** Kernel dynamically allocates memory without leaking or page faulting on valid access.
- **Measurable Benchmark:** Page allocation latency.
- **Risks:** Memory corruption, fragmentation.
- **Complexity:** High.

## M3: Scheduler (workload-aware)
- **Objective:** Preemptive multitasking in kernel space.
- **Dependencies:** M2, APIC/Timer configuration.
- **Implementation Tasks:** Thread representation, context switching, basic round-robin scheduler, APIC timer interrupts.
- **Tests:** Run three kernel threads printing alternating patterns.
- **Acceptance Criteria:** Threads preempt each other reliably without data corruption.
- **Measurable Benchmark:** Context switch overhead (cycles).
- **Risks:** Register state corruption during context switch.
- **Complexity:** High.

## M4: Processes + Threads
- **Objective:** Ring 3 execution.
- **Dependencies:** M3.
- **Implementation Tasks:** User-mode transition (TSS, SYSCALL/SYSRET), virtual address space isolation per process.
- **Tests:** Execute a dummy statically compiled Ring 3 payload.
- **Acceptance Criteria:** User space executes; CPU exceptions in user space do not crash the kernel.
- **Measurable Benchmark:** Ring 3 to Ring 0 transition time.
- **Risks:** Security vulnerabilities in ring transition.
- **Complexity:** Medium.

## M5: Syscalls + IPC
- **Objective:** Communication between user space and kernel/servers.
- **Dependencies:** M4.
- **Implementation Tasks:** Syscall dispatcher, asynchronous IPC, message passing primitives.
- **Tests:** User process requests kernel to print a string via IPC.
- **Acceptance Criteria:** IPC messages delivered accurately and securely.
- **Measurable Benchmark:** IPC round-trip time.
- **Risks:** IPC bottlenecks leading to system-wide latency.
- **Complexity:** High.

## M6: CatRAM + VFS
- **Objective:** In-memory filesystem for bootstrapping user-space.
- **Dependencies:** M5.
- **Implementation Tasks:** VFS server in user-space, CatRAM implementation.
- **Tests:** Create, read, and delete files in CatRAM.
- **Acceptance Criteria:** Concurrent access to VFS is thread-safe; RAM disk works.
- **Measurable Benchmark:** VFS path resolution speed.
- **Risks:** Concurrency bugs in user-space VFS.
- **Complexity:** Medium.

## M7: Storage + Basic Drivers
- **Objective:** Read/write capability to block devices.
- **Dependencies:** M6, PCI enumeration.
- **Implementation Tasks:** PCI bus scanning, NVMe/AHCI block driver, storage abstraction layer.
- **Tests:** Read Sector 0 of an attached QEMU block device.
- **Acceptance Criteria:** Reliable sector-level read/write.
- **Measurable Benchmark:** IOPS on raw block device.
- **Risks:** Hardware diversity, DMA complexity.
- **Complexity:** Medium.

## M8: Security + Sandbox
- **Objective:** Capability-based security and workload isolation.
- **Dependencies:** M7.
- **Implementation Tasks:** Process Manager, capabilities assignment, strict resource sandboxing.
- **Tests:** Attempt privilege escalation from a confined user process.
- **Acceptance Criteria:** Processes cannot access unauthorized resources or IPC endpoints.
- **Measurable Benchmark:** Privilege check overhead.
- **Risks:** Sandbox escape vectors.
- **Complexity:** High.

## M9: Graphics + Compositor
- **Objective:** Visual output and GUI foundation.
- **Dependencies:** M8.
- **Implementation Tasks:** Virtio-gpu/UEFI GOP, user-space compositor, basic input routing.
- **Tests:** Render overlapping windows.
- **Acceptance Criteria:** Tearing-free 60fps window rendering.
- **Measurable Benchmark:** Compositor frame latency.
- **Risks:** Driver models for real GPUs.
- **Complexity:** High.

## M10: Desktop Shell
- **Objective:** Core Catalyst user interface.
- **Dependencies:** M9.
- **Implementation Tasks:** Window manager, taskbar, settings app.
- **Tests:** Launch apps, resize windows, navigate UI.
- **Acceptance Criteria:** Usable, responsive desktop environment.
- **Measurable Benchmark:** UI responsiveness (input-to-frame delay).
- **Risks:** UX friction.
- **Complexity:** Medium.

## M11: Developer Platform
- **Objective:** Toolchain and SDK readiness.
- **Dependencies:** M10.
- **Implementation Tasks:** Catalyst SDK (Rust/C++), compiler toolchains, self-hosting ability.
- **Tests:** Build Catalyst on Catalyst.
- **Acceptance Criteria:** Successful compilation of hello world via SDK.
- **Measurable Benchmark:** Compilation time vs host OS.
- **Risks:** POSIX incompatibilities for LLVM.
- **Complexity:** High.

## M12: Immutable OS + Updates
- **Objective:** Robust system updates using snapshots.
- **Dependencies:** M11, CatFS v0/Production.
- **Implementation Tasks:** A/B partitions or filesystem snapshots, OTA update daemon.
- **Tests:** Apply update, simulate power loss, verify rollback.
- **Acceptance Criteria:** Atomic system updates with guaranteed rollback on failure.
- **Measurable Benchmark:** Update application time.
- **Risks:** Brick risk during updates.
- **Complexity:** High.

## M13: Compatibility Runtimes
- **Objective:** Win32, Linux, and Android user-space translation layers.
- **Dependencies:** M12.
- **Implementation Tasks:** User-space syscall translators, PE/ELF/APK loaders.
- **Tests:** Run basic Linux coreutils and headless Windows EXEs.
- **Acceptance Criteria:** Foreign binaries execute without kernel modification.
- **Measurable Benchmark:** Syscall translation overhead.
- **Risks:** Massive API surface area.
- **Complexity:** High.

## M14: Gaming
- **Objective:** Support high-end PC gaming via Windows translation.
- **Dependencies:** M13.
- **Implementation Tasks:** DXVK/Vulkan translation, input controller mapping, audio subsystems.
- **Tests:** Launch a heavy 3D Steam game.
- **Acceptance Criteria:** Playable frame rates on equivalent hardware to Windows.
- **Measurable Benchmark:** FPS and 1% lows in benchmark titles.
- **Risks:** Anti-cheat incompatibilities.
- **Complexity:** High.

## M15: AI Subsystem
- **Objective:** Native hardware-accelerated AI inference.
- **Dependencies:** M14.
- **Implementation Tasks:** NPU/GPU tensor execution abstractions, OS-level ML models for scheduler/search.
- **Tests:** Run local LLM inference via Catalyst API.
- **Acceptance Criteria:** Offloaded inference does not stutter the compositor.
- **Measurable Benchmark:** Tokens per second on NPU.
- **Risks:** Fragmented NPU drivers.
- **Complexity:** High.

## M16: Hardware Validation
- **Objective:** Real hardware deployment.
- **Dependencies:** M15.
- **Implementation Tasks:** Bare-metal testing on reference x86-64 hardware.
- **Tests:** Boot from USB on physical laptop/desktop.
- **Acceptance Criteria:** System operates stably with WiFi, audio, and graphics.
- **Measurable Benchmark:** Battery life and thermal performance.
- **Risks:** Driver gaps for obscure hardware.
- **Complexity:** High.

## M17: Catalyst OS 1.0 Candidate
- **Objective:** Final polish for 1.0 release.
- **Dependencies:** M16.
- **Implementation Tasks:** Bug fixes, documentation, ISO mastering.
- **Tests:** Public beta testing.
- **Acceptance Criteria:** ARB sign-off for 1.0 release.
- **Measurable Benchmark:** MTBF (Mean Time Between Failures).
- **Risks:** Critical zero-days.
- **Complexity:** Medium.
