<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS: Development Roadmap v0.1

## Core Philosophy
Core OS correctness precedes all filesystem and compatibility work. Development progresses linearly through foundational components.

---

## Milestone 0: Boot & Execution Environment
- **Objective:** Successfully hand off from UEFI to Rust kernel environment.
- **Dependencies:** None.
- **Implementation Tasks:** UEFI bootloader, kernel ELF loading, long mode initialization, basic serial logging, global descriptor table (GDT), interrupt descriptor table (IDT).
- **Tests:** Boot QEMU, catch a divide-by-zero exception.
- **Acceptance Criteria:** Kernel boots, prints to serial, handles basic CPU exceptions safely.
- **Measurable Benchmark:** Boot time (UEFI to `kmain`) < 50ms.
- **Risks:** Toolchain configuration issues, UEFI idiosyncrasies.
- **Complexity:** Low

## Milestone 1: Physical & Virtual Memory Management
- **Objective:** Establish safe, dynamic memory allocation.
- **Dependencies:** M0.
- **Implementation Tasks:** Physical frame allocator (bitmap or buddy), page table management, kernel heap allocator.
- **Tests:** Allocate and free random blocks; stress test page mapping/unmapping.
- **Acceptance Criteria:** Kernel can dynamically allocate memory without leaking or page faulting on valid access.
- **Measurable Benchmark:** Page allocation latency.
- **Risks:** Hard-to-debug memory corruption, fragmentation.
- **Complexity:** High

## Milestone 2: Kernel Task & Scheduler Foundation
- **Objective:** Preemptive multitasking in kernel space.
- **Dependencies:** M1, APIC/Timer configuration.
- **Implementation Tasks:** Thread representation, context switching (NASM), basic round-robin scheduler, APIC timer interrupts.
- **Tests:** Run three kernel threads printing alternating patterns.
- **Acceptance Criteria:** Threads preempt each other reliably without data corruption.
- **Measurable Benchmark:** Context switch overhead (cycles).
- **Risks:** Register state corruption during context switch.
- **Complexity:** High

## Milestone 3: User-Space & Process Model
- **Objective:** Ring 3 execution.
- **Dependencies:** M2.
- **Implementation Tasks:** User-mode transition (TSS, SYSCALL/SYSRET), virtual address space isolation per process.
- **Tests:** Execute a dummy statically compiled Ring 3 payload.
- **Acceptance Criteria:** User space executes; CPU exceptions in user space do not crash the kernel.
- **Measurable Benchmark:** Ring 3 to Ring 0 transition time.
- **Risks:** Security vulnerabilities in ring transition.
- **Complexity:** Medium

## Milestone 4: Syscall Interface & IPC Primitive
- **Objective:** Communication between user space and kernel/servers.
- **Dependencies:** M3.
- **Implementation Tasks:** Syscall dispatcher, basic synchronous IPC (message passing).
- **Tests:** User process requests kernel to print a string via IPC.
- **Acceptance Criteria:** IPC messages delivered accurately and securely.
- **Measurable Benchmark:** IPC round-trip time.
- **Risks:** IPC bottlenecks leading to system-wide latency.
- **Complexity:** High

## Milestone 5: Basic Storage & Device Driver Model
- **Objective:** Read/write capability to block devices.
- **Dependencies:** M4, PCI enumeration.
- **Implementation Tasks:** PCI bus scanning, AHCI or NVMe basic driver, block device interface.
- **Tests:** Read Sector 0 of an attached QEMU block device.
- **Acceptance Criteria:** Reliable sector-level read/write without blocking the whole system.
- **Measurable Benchmark:** IOPS on raw block device.
- **Risks:** Hardware diversity, DMA complexity.
- **Complexity:** Medium

## Milestone 6: Virtual File System (VFS)
- **Objective:** Abstract representation of files and directories.
- **Dependencies:** M5.
- **Implementation Tasks:** VFS core, mount points, file descriptors, `tmpfs`.
- **Tests:** Create, read, and delete files in RAM (`tmpfs`).
- **Acceptance Criteria:** Concurrent access to VFS is thread-safe.
- **Measurable Benchmark:** VFS path resolution speed.
- **Risks:** Concurrency bugs, lock contention.
- **Complexity:** Medium

## Milestone 7: Catalyst FS v0 (Persistent Storage)
- **Objective:** A basic, custom persistent filesystem.
- **Dependencies:** M5, M6.
- **Implementation Tasks:** Superblock, inodes, directory entries, basic journaling/crash consistency.
- **Tests:** Write file, reboot QEMU, read file back.
- **Acceptance Criteria:** Data persists across reboots without corruption.
- **Measurable Benchmark:** Sequential and random I/O throughput.
- **Risks:** Data loss on unexpected shutdown.
- **Complexity:** High

## Milestone 8: ELF Loader & Dynamic Linking
- **Objective:** Load complex native binaries.
- **Dependencies:** M6.
- **Implementation Tasks:** ELF parser, dynamic linker (`ld.so` equivalent).
- **Tests:** Compile and run a standard Rust user-space binary dynamically linked to a libc/libcatalyst.
- **Acceptance Criteria:** Accurate relocation and symbol resolution.
- **Measurable Benchmark:** Binary load time.
- **Risks:** ABI incompatibilities.
- **Complexity:** Medium

## Milestone 9: Initial Service Ecosystem (User-space OS)
- **Objective:** Move core OS features out of the kernel.
- **Dependencies:** M4, M8.
- **Implementation Tasks:** Process Manager (PM), VFS Server, Device Manager running in Ring 3.
- **Tests:** Boot system where kernel only provides IPC and scheduling; everything else is in servers.
- **Acceptance Criteria:** System remains stable under heavy IPC load.
- **Measurable Benchmark:** System call overhead via user-space servers vs monolithic.
- **Risks:** Microkernel performance penalty.
- **Complexity:** High

## Milestone 10: Networking Stack (TCP/IP)
- **Objective:** Internet connectivity.
- **Dependencies:** M9.
- **Implementation Tasks:** Virtio-net driver, user-space TCP/IP stack (e.g., smoltcp integration).
- **Tests:** Ping, basic HTTP GET.
- **Acceptance Criteria:** Reliable packet transmission and reception.
- **Measurable Benchmark:** Network throughput (Gbps).
- **Risks:** Network stack vulnerabilities.
- **Complexity:** Medium

## Milestone 11: Graphics & Window Compositor
- **Objective:** Visual output and GUI foundation.
- **Dependencies:** M9.
- **Implementation Tasks:** UEFI GOP driver / Virtio-gpu, user-space compositor, basic input routing (keyboard/mouse).
- **Tests:** Render multiple overlapping windows.
- **Acceptance Criteria:** Tearing-free 60fps window rendering.
- **Measurable Benchmark:** Compositor frame latency.
- **Risks:** Complex driver models for real GPUs.
- **Complexity:** High

## Milestone 12: Toolchain & Self-Hosting Prep
- **Objective:** Compile Catalyst on Catalyst.
- **Dependencies:** M7, M10.
- **Implementation Tasks:** Port Rust compiler (rustc), LLVM, Cargo.
- **Tests:** Run `cargo build` inside Catalyst.
- **Acceptance Criteria:** Successful compilation of a Hello World Rust app on Catalyst.
- **Measurable Benchmark:** Compilation time vs host OS.
- **Risks:** Missing POSIX/libc dependencies blocking LLVM.
- **Complexity:** High

## Milestone 13: Linux Compatibility Layer (Basic)
- **Objective:** Run simple Linux tools.
- **Dependencies:** M12.
- **Implementation Tasks:** Syscall translator (read, write, open, mmap).
- **Tests:** Run unmodified Linux `ls` and `grep`.
- **Acceptance Criteria:** Tools work as expected without recompilation.
- **Measurable Benchmark:** Syscall translation overhead.
- **Risks:** Divergent FS semantics.
- **Complexity:** Medium

## Milestone 14: Windows Compatibility Layer (Basic)
- **Objective:** Run basic headless Windows exes.
- **Dependencies:** M8.
- **Implementation Tasks:** PE loader, fundamental Win32 API stubs (kernel32.dll).
- **Tests:** Run a basic Win32 console application.
- **Acceptance Criteria:** Application runs and prints to standard output.
- **Measurable Benchmark:** Execution initialization time.
- **Risks:** Overwhelming API surface.
- **Complexity:** Medium

## Milestone 15: Public Developer Preview (v0.1)
- **Objective:** Release testable ISO to developers.
- **Dependencies:** M0-M14.
- **Implementation Tasks:** ISO generation, installer, documentation, SDK packaging.
- **Tests:** End-to-end install and application development cycle.
- **Acceptance Criteria:** A developer can install OS, write a native app, and run it.
- **Measurable Benchmark:** Installer success rate.
- **Risks:** Critical bugs affecting initial public perception.
- **Complexity:** High
