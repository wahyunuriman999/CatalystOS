<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS Performance Contract v0.1

This document defines the measurable performance benchmarks and engineering targets for Catalyst OS. These are not marketing claims, but rigorous engineering aspirations and invariants to guide architectural decisions.

## 1. Memory

**What to measure (specific metric):**
- Kernel footprint (base kernel + core drivers)
- Desktop footprint (compositor + essential user-space servers)
- Service footprint (background daemons, RPC servers)
- Runtime overhead per application process
- Cache efficiency (page cache hit rate, eviction latency)
- Memory pressure behavior (time-to-OOM, swap/compression latency)

**How to measure (tool/methodology):**
- Automated QEMU microVM tests reading physical memory allocation.
- In-kernel memory profiling and tracing (`cat_mem_trace`).
- Stress testing with simulated memory limits (cgroups equivalent).

**Target range (aspirational but grounded):**
- Kernel footprint: < 16MB base, < 32MB with core drivers.
- Desktop idle (total system): < 500MB (Aspirational goal).
- Process overhead: < 1MB per basic user-space process.

**Why this target matters:**
A low base footprint leaves more memory for user applications and file caching. Low process overhead is critical for a hybrid microkernel where many system services run as separate user-space processes.

## 2. Boot

**What to measure (specific metric):**
- Firmware-to-bootloader: UEFI handover time.
- Bootloader-to-kernel: Kernel load and decompression time.
- Kernel init: Basic CPU and memory setup, scheduler readiness.
- Service init: User-space server launch (VFS, device manager).
- Desktop readiness: Time to first interactive frame in compositor.

**How to measure (tool/methodology):**
- UEFI timestamps.
- CPU cycle counters (RDTSC) recorded in kernel early boot.
- Hardware-assisted tracing via QEMU profiling.

**Target range (aspirational but grounded):**
- Kernel init: < 100ms.
- Service init: < 200ms (via parallel and lazy initialization).
- Desktop readiness: < 1 second on NVMe SSD.

**Why this target matters:**
Fast boot times improve user perception and are essential for rapid development iteration and embedded/appliance use-cases.

## 3. Responsiveness

**What to measure (specific metric):**
- Input latency: Hardware interrupt to compositor event delivery.
- Compositor latency: Event reception to frame presentation.
- Frame pacing: Variance in frame intervals.
- App launch: Click to first window render.
- Context switching: User-space to kernel-space and IPC overhead.
- UI under CPU/memory pressure: Frame drops during compilation/stress.

**How to measure (tool/methodology):**
- High-speed camera/hardware latency testers (e.g., LDAT).
- IPC microbenchmarks.
- Tracing tools for compositor frame pipelines.

**Target range (aspirational but grounded):**
- Input latency: < 5ms (interrupt to user-space).
- IPC context switch: < 1us.
- Frame pacing: 99.9% frames within 16.6ms (60Hz) or 8.3ms (120Hz).
- App launch (cached): < 50ms for basic apps.

**Why this target matters:**
Responsiveness defines the subjective "feel" of the OS. Low IPC latency is the make-or-break metric for microkernel architectures.

## 4. Gaming

**What to measure (specific metric):**
- FPS and 1% lows.
- Frame-time variance (stutter).
- Input latency during high GPU load.
- Shader compilation impact.
- CPU overhead for graphics APIs.
- GPU utilization and thermal throttling behavior.

**How to measure (tool/methodology):**
- Standardized game benchmark loops.
- Frame time capture tools (e.g., MangoHud equivalent).

**Target range (aspirational but grounded):**
- 1% lows within 10% of average FPS.
- CPU overhead: < 5% penalty compared to bare-metal Windows/Linux.
- Zero added frame pacing jitter from background OS tasks.

**Why this target matters:**
Gaming stresses the entire stack: scheduler, memory management, graphics drivers, and input. Success here validates the core architecture's efficiency.

## 5. Developer Workloads

**What to measure (specific metric):**
- Compilation throughput (e.g., compiling the kernel itself).
- Linker performance (high memory and I/O burst).
- File system metadata operations (stat, readdir).
- Git operations (status, checkout).
- Container/Sandboxing performance (startup time, network overhead).
- IDE and LSP responsiveness.

**How to measure (tool/methodology):**
- Scripted builds (Rust `cargo build`, C `make -j`).
- Synthetic metadata benchmarks (file creation/deletion loops).

**Target range (aspirational but grounded):**
- FS metadata ops: > 100,000 ops/sec.
- Process creation: < 500us.
- Compilation time: Parity or better than Linux ext4/tmpfs.

**Why this target matters:**
Catalyst OS is self-hosting; developer experience dictates engineering velocity. Fast metadata operations and process creation are essential for build systems and version control.

## Benchmark Suite Strategy

To ensure these targets are met across milestones:
1. **Automated CI Microbenchmarks:** IPC latency, memory footprint, and boot time are measured on every PR using QEMU.
2. **Nightly Macrobenchmarks:** Compilation times, FS metadata ops, and complex service launches run nightly on bare-metal CI runners.
3. **Release Profiling:** Full gaming and responsiveness suites run manually or semi-automatically before minor/major version releases.
4. **Regression Alerts:** Any deviation >5% from baseline triggers an automatic failure.
