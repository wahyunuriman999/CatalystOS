<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS Performance Contract v0.2

This document defines the measurable performance benchmarks and engineering targets for Catalyst OS. 

**CRITICAL NOTE:** Metrics such as `<500MB RAM` and `2-3 seconds boot time` are engineering **TARGETS**, not strict invariants or marketing promises. They serve as goals to benchmark against during development across various hardware configurations.

## 1. Hardware Profiles

To accurately measure performance, we define four distinct hardware profiles:

- **Profile A (Entry-level):** 4GB RAM, dual-core CPU, integrated GPU, eMMC/SATA SSD
- **Profile B (Mainstream laptop):** 8-16GB RAM, quad-core CPU, integrated GPU, NVMe
- **Profile C (Developer workstation):** 32-64GB RAM, 8+ cores CPU, dedicated GPU, NVMe
- **Profile D (Gaming desktop):** 16-32GB RAM, 8+ cores CPU, high-end GPU, NVMe

## 2. Benchmark Categories & Targets

For each category and profile, we define specific measurable targets. Measurement methodology involves automated QEMU microVM tests, hardware cycle counters, and strict CI regression tracking.

### Catalyst Minimal (Kernel + Console Only)
**Target Focus:** Bare-metal efficiency, fast boot.
- **Profile A:** Boot < 2s, RAM < 64MB
- **Profile B, C, D:** Boot < 1s, RAM < 64MB
- **Methodology:** QEMU minimal init tests; UEFI timestamps to console prompt.

### Catalyst Desktop (Full Desktop, Idle)
**Target Focus:** Everyday user baseline, background resource usage.
- **Profile A:** Boot < 5s, RAM < 500MB
- **Profile B:** Boot < 3s, RAM < 600MB
- **Profile C, D:** Boot < 2s, RAM < 800MB (accounting for complex GPU drivers)
- **Methodology:** Time to first interactive frame; physical memory footprint at 60s idle.

### Catalyst Developer (IDE + Terminal + Compiler)
**Target Focus:** Sustained IO, parallel processing, context switching.
- **Profile B (Mainstream):** FS metadata > 50k ops/sec, compile time parity with Linux.
- **Profile C (Workstation):** FS metadata > 150k ops/sec, instantaneous IDE indexing.
- **Methodology:** Automated kernel build scripts, synthetic FS metadata benchmarks.

### Catalyst Gaming (Game Running + Background Minimal)
**Target Focus:** GPU utilization, input latency, frame pacing.
- **Profile B (Mainstream):** 1% lows within 15% of average FPS.
- **Profile D (Gaming Desktop):** 1% lows within 5% of average FPS; 0 added frame pacing jitter.
- **Methodology:** Standardized benchmark loops with frame time capture.

### Catalyst Compatibility (Win32 Runtime Loaded)
**Target Focus:** Translation overhead, memory sharing.
- **Profile A, B, C, D:** Translation overhead < 10% CPU penalty vs native Windows; runtime memory overhead < 150MB base.
- **Methodology:** IPC microbenchmarks passing through the compatibility layer; Windows game benchmarks.

## 3. Core Measurement Domains (Methodology Details)

### Memory
- **What:** Kernel footprint, Desktop footprint, process overhead, cache efficiency.
- **How:** In-kernel memory profiling (`cat_mem_trace`), QEMU microVM tests.
- **Target Context:** Low base footprint leaves more memory for user applications and file caching. Process overhead < 1MB per basic user-space process.

### Boot
- **What:** Firmware-to-bootloader, Kernel init, Service init, Desktop readiness.
- **How:** UEFI timestamps, RDTSC cycle counters.
- **Target Context:** Kernel init < 100ms. Fast boot times improve user perception and iteration speed.

### Responsiveness
- **What:** Input latency, compositor latency, frame pacing, IPC context switch overhead.
- **How:** High-speed camera (LDAT), IPC microbenchmarks.
- **Target Context:** Input latency < 5ms. IPC context switch < 1us. Critical for hybrid microkernel architecture.

### Gaming
- **What:** FPS, 1% lows, input latency under GPU load, CPU overhead for graphics APIs.
- **How:** Frame time capture tools, benchmark loops.
- **Target Context:** Zero added frame pacing jitter from background tasks. Validates the core architecture.

### Developer Workloads
- **What:** Compilation throughput, linker performance, FS metadata operations, process creation time.
- **How:** Scripted builds, synthetic metadata tests.
- **Target Context:** Fast metadata ops and process creation are essential for developer velocity on a self-hosting OS.

## 4. Benchmark Suite Strategy
1. **Automated CI Microbenchmarks:** IPC latency, memory footprint, and boot time are measured on every PR.
2. **Nightly Macrobenchmarks:** Compilation times, FS metadata ops run nightly on bare-metal CI.
3. **Release Profiling:** Full gaming and responsiveness suites run manually before releases.
4. **Regression Alerts:** Any deviation >5% from baseline triggers an automatic failure.
