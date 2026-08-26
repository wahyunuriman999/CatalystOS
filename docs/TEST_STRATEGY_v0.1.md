<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS Test Strategy v0.1

This document defines the testing strategy for Catalyst OS, a hybrid microkernel operating system written in Rust. Our goal is to ensure stability, security, and performance across all layers of the OS stack.

## 1. Unit Testing
*   **What we test:** Individual Rust modules, memory allocators, data structures, and isolated logic within the kernel and user-space servers.
*   **How we test:** Using Rust's built-in `#[test]` framework.
*   **Tools:** `cargo test`.
*   **Automation:** Run automatically on every commit in the CI pipeline.
*   **Pass/Fail Criteria:** All tests must pass, 90%+ code coverage for core data structures.

## 2. Integration Testing
*   **What we test:** Interactions between subsystems (e.g., VFS and block device drivers, IPC mechanisms between servers).
*   **How we test:** QEMU-based boot tests with custom test harnesses that run specific integration scenarios.
*   **Tools:** QEMU, custom Rust test runner capturing serial output.
*   **Automation:** Nightly CI runs and pre-merge checks for critical subsystems.
*   **Pass/Fail Criteria:** Expected serial output matches actual output, no kernel panics.

## 3. Boot Testing
*   **What we test:** The OS's ability to successfully boot via UEFI, initialize the kernel, and drop into a functional shell or user space.
*   **How we test:** Automated QEMU execution with a timeout.
*   **Tools:** QEMU, OVMF (UEFI firmware), Python expect scripts (e.g., `pexpect` or similar serial parsers).
*   **Automation:** Required for every PR merge.
*   **Pass/Fail Criteria:** System reaches the "Ready" prompt within the defined timeout (e.g., < 5 seconds) without errors in the boot log.

## 4. Syscall Testing
*   **What we test:** Conformance, correctness, and security of the system call interface.
*   **How we test:** A dedicated userspace test suite that exercises all syscalls with valid, invalid, and edge-case arguments.
*   **Tools:** Custom C/Rust test suite (similar to LTP for Linux).
*   **Automation:** Nightly CI runs.
*   **Pass/Fail Criteria:** All tests pass, invalid inputs return correct error codes without crashing the kernel.

## 5. Memory Testing
*   **What we test:** Allocation/deallocation stress, OOM (Out Of Memory) behavior, page fault handling, memory leak detection.
*   **How we test:** Stress tests running in userspace and kernel space; memory thrashing scenarios.
*   **Tools:** Custom memory stress tools, Valgrind (for userspace servers, if ported), Rust's miri (for host-tested allocators).
*   **Automation:** Weekly deep-dive test runs.
*   **Pass/Fail Criteria:** System gracefully handles OOM (kills appropriate processes) without kernel panic, no detected leaks over 24-hour stress runs.

## 6. Filesystem Testing
*   **What we test:** Crash consistency, corruption recovery, stress testing of the custom FS or adopted FS (e.g., ext2/FAT32 for boot).
*   **How we test:** Sudden power loss simulation (killing QEMU), heavy concurrent read/write workloads.
*   **Tools:** `fsck` equivalents, customized `fsstress`.
*   **Automation:** Nightly runs.
*   **Pass/Fail Criteria:** Filesystem mounts successfully after simulated crashes; no data corruption in committed transactions.

## 7. Scheduler Testing
*   **What we test:** Fairness, priority inversion prevention (e.g., priority inheritance), latency measurement.
*   **How we test:** Spawning hundreds of threads with varying priorities; measuring context switch times.
*   **Tools:** Custom benchmarking utilities using high-resolution timers.
*   **Automation:** Performance regressions tracked per-commit.
*   **Pass/Fail Criteria:** Context switch overhead remains under budget (e.g., < 2µs on target hardware); high-priority threads are not starved.

## 8. Security Testing
*   **What we test:** Privilege escalation attempts, sandbox escape, capability system integrity.
*   **How we test:** Exploitation scripts attempting to bypass IPC restrictions or access unauthorized memory.
*   **Tools:** Custom exploit payloads.
*   **Automation:** Continuous integration.
*   **Pass/Fail Criteria:** All exploits fail to escalate privileges or crash the kernel.

## 9. Performance Testing
*   **What we test:** IPC latency, boot time, memory footprint, filesystem throughput.
*   **How we test:** Automated benchmarking suite tied to the Performance Contract.
*   **Tools:** Criterion.rs (host), custom OS-level benchmarking tools.
*   **Automation:** Post-merge CI pipeline.
*   **Pass/Fail Criteria:** Performance metrics do not regress beyond a 5% margin compared to the baseline.

## 10. Compatibility Testing
*   **What we test:** Application compatibility matrix (POSIX/Win32 subsystems).
*   **How we test:** Running a predefined suite of unmodified third-party binaries.
*   **Tools:** Custom wrapper scripts.
*   **Automation:** Nightly.
*   **Pass/Fail Criteria:** Critical applications in the matrix execute and function correctly.

## 11. Hardware Testing
*   **What we test:** Real hardware validation.
*   **How we test:** Deploying the OS image to physical x86-64 machines via PXE or USB.
*   **Tools:** LAVA (Linaro Automated Validation Architecture) or custom PXE boot infrastructure.
*   **Automation:** Milestone releases (M14+).
*   **Pass/Fail Criteria:** System boots and operates stably on designated target hardware.

## 12. Regression Testing
*   **What we test:** Previously fixed bugs and established functionality.
*   **How we test:** Re-running the entire suite (unit, integration, boot).
*   **Tools:** CI Pipeline.
*   **Automation:** Every commit.
*   **Pass/Fail Criteria:** Zero regressions.

## 13. Fuzzing
*   **What we test:** Syscall interfaces, filesystem parsers, network stack.
*   **How we test:** Feeding random/mutated inputs to kernel entry points.
*   **Tools:** Syzkaller (adapted for Catalyst OS), `cargo fuzz` / libFuzzer.
*   **Automation:** Continuous background fuzzing cluster.
*   **Pass/Fail Criteria:** Zero panics or deadlocks discovered over a 72-hour fuzzing window.

## CI/CD Pipeline Architecture
1.  **Commit/PR Stage:**
    *   Linting (`cargo clippy`, `cargo fmt`).
    *   Unit Tests (`cargo test` on host).
    *   Build OS Image (`cargo build --target x86_64-catalyst_os.json`).
    *   Boot Test (QEMU, assert on serial output).
2.  **Nightly Stage:**
    *   Integration Tests.
    *   Syscall Conformance Tests.
    *   Performance Benchmarking.
    *   Filesystem Stress Tests.
3.  **Release Stage (Milestones):**
    *   Full compatibility matrix run.
    *   Real hardware deployment (automated PXE).
    *   Security and Fuzzing sign-off.
