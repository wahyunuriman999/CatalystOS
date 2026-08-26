<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS Test Strategy v0.2

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

## 3. Kernel Architecture Test Requirements
*   **What we test:** Validation of the hybrid microkernel model.
*   **How we test:** Verifying that kernel panics in user-space servers (e.g., VFS, Process Manager) do not bring down the core kernel, and that the kernel can safely restart crashed servers without system compromise.
*   **Tools:** Fault injection frameworks targeting user-space services.
*   **Automation:** Nightly runs.
*   **Pass/Fail Criteria:** The core kernel remains responsive and isolates faults when a critical user-space server crashes.

## 4. GPU Split-Architecture Testing
*   **What we test:** The boundary between the minimal kernel graphics driver (e.g., memory management, interrupts) and the complex user-space graphics component (compositor).
*   **How we test:** Stressing the IPC bridge between the user-space graphics server and the kernel driver.
*   **Tools:** Automated UI frame-buffer validators, fuzzing the GPU IPC endpoint.
*   **Automation:** Continuous integration.
*   **Pass/Fail Criteria:** Invalid GPU commands from user-space applications are caught and sandboxed; the compositor does not crash the kernel.

## 5. Compatibility Runtime Isolation Testing
*   **What we test:** Ensuring Windows, Linux, and Android runtimes are strictly contained.
*   **How we test:** Running malicious payloads designed for Windows/Linux against the translation layer to verify they cannot escape into the native Catalyst environment.
*   **Tools:** Standard exploit test suites (e.g., Metasploit payloads for Win32/ELF).
*   **Automation:** Security pipeline.
*   **Pass/Fail Criteria:** Malicious foreign code is perfectly sandboxed within the compatibility runtime's capabilities.

## 6. Catalyst Core Purity Verification Tests
*   **What we test:** Enforcement of the Catalyst Core Purity principle (compatibility requirements must never redefine the fundamental architecture of the kernel).
*   **How we test:** Static analysis and CI checks ensuring no API endpoints or kernel primitives are merged if they are specific to only one compatibility subsystem (e.g., checking for Windows-specific anti-cheat stubs in the kernel).
*   **Tools:** Custom `cargo clippy` lints and architectural boundary enforcement scripts.
*   **Automation:** Required for every PR merge.
*   **Pass/Fail Criteria:** The build fails if a kernel PR includes compatibility-specific hacks rather than generic primitives.

## 7. Performance Regression Testing
*   **What we test:** IPC latency, boot time, memory footprint, filesystem throughput tied to the Performance Contract.
*   **How we test:** Automated benchmarking suite that profiles specific operations (e.g., context switches, syscalls) and compares them against strict performance contract profiles.
*   **Tools:** Criterion.rs (host), custom OS-level benchmarking tools.
*   **Automation:** Post-merge CI pipeline.
*   **Pass/Fail Criteria:** Performance metrics do not regress beyond a 5% margin compared to the baseline contract.

## 8. Boot Testing
*   **What we test:** The OS's ability to successfully boot via UEFI, initialize the kernel, and drop into a functional shell or user space.
*   **How we test:** Automated QEMU execution with a timeout.
*   **Tools:** QEMU, OVMF (UEFI firmware), Python expect scripts.
*   **Automation:** Required for every PR merge.
*   **Pass/Fail Criteria:** System reaches the "Ready" prompt within the defined timeout (e.g., < 5 seconds).

## 9. Syscall Testing
*   **What we test:** Conformance, correctness, and security of the system call interface.
*   **Tools:** Custom C/Rust test suite.
*   **Automation:** Nightly CI runs.
*   **Pass/Fail Criteria:** Invalid inputs return correct error codes without crashing the kernel.

## 10. Memory Testing
*   **What we test:** Allocation/deallocation stress, OOM behavior, page fault handling, memory leak detection.
*   **Tools:** Valgrind (if ported), Rust's miri, custom memory stress tools.
*   **Automation:** Weekly deep-dive test runs.
*   **Pass/Fail Criteria:** System gracefully handles OOM; no detected leaks over 24-hour stress runs.

## 11. Filesystem Testing
*   **What we test:** Crash consistency, corruption recovery, stress testing of CatRAM/CatFS.
*   **Tools:** `fsck` equivalents, customized `fsstress`.
*   **Automation:** Nightly runs.
*   **Pass/Fail Criteria:** Filesystem mounts successfully after simulated crashes; no data corruption.

## 12. Scheduler Testing
*   **What we test:** Fairness, latency measurement, priority inversion prevention.
*   **Tools:** Custom benchmarking utilities using high-resolution timers.
*   **Automation:** Performance regressions tracked per-commit.
*   **Pass/Fail Criteria:** Context switch overhead remains under budget; high-priority threads are not starved.

## 13. Security Testing
*   **What we test:** Privilege escalation, capability system integrity, sandbox escape.
*   **Tools:** Custom exploit payloads.
*   **Automation:** Continuous integration.
*   **Pass/Fail Criteria:** Exploits fail to escalate privileges.

## 14. Compatibility Testing
*   **What we test:** Application compatibility matrix.
*   **Tools:** Custom wrapper scripts.
*   **Automation:** Nightly.
*   **Pass/Fail Criteria:** Critical applications execute correctly.

## 15. Hardware Testing
*   **What we test:** Real hardware validation.
*   **Tools:** LAVA or custom PXE boot infrastructure.
*   **Automation:** Milestone releases (M14+).
*   **Pass/Fail Criteria:** System boots and operates stably on designated target hardware.

## 16. Fuzzing
*   **What we test:** Syscall interfaces, filesystem parsers, network stack.
*   **Tools:** Syzkaller, `cargo fuzz` / libFuzzer.
*   **Automation:** Continuous background fuzzing cluster.
*   **Pass/Fail Criteria:** Zero panics or deadlocks discovered over 72-hour window.
