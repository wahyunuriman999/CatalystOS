<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS: Compatibility Strategy v0.2

## Core Philosophy

Catalyst must be a **native OS FIRST** and a compatibility platform **SECOND**. Compatibility runtimes must be strictly isolated from the core OS architecture to ensure system stability, security, and maintainability.

### CRITICAL NEW PRINCIPLE — Catalyst Core Purity
**Compatibility requirements must never redefine the fundamental architecture of Catalyst Kernel.**

When an app needs a kernel primitive:
1. Is it genuinely needed generally? → Add as Catalyst-native API
2. Can it be provided by user-space service? → Implement in compatibility runtime
3. Does it require privilege? → Create a small GENERIC kernel primitive
4. Is it useful for Catalyst native apps too? → If yes, add to kernel. If only Windows needs it → user-space only.
5. 'Because Windows needs it' is NOT sufficient justification for kernel changes.

### Architecture
```text
             Catalyst Core
                  │
    ┌─────────────┼─────────────┐
    ↓             ↓             ↓
Catalyst       Windows       Android
 Native        Runtime        Runtime
    │             │             │
    └─────────────┼─────────────┘
                  ↓
            Catalyst APIs
                  ↓
            Catalyst Kernel
```

## 1. Native Catalyst Ecosystem
**Objective:** Establish a robust, high-performance native application environment.
- **Requirements:** Safe memory management (Rust), asynchronous IPC, capability-based security.
- **Technical Approach:** Catalyst SDK (Rust primary, C/C++ bindings). Strict capability enforcement. Component-based application model.
- **Legal Considerations:** Permissive licensing (MIT/Apache 2.0) for standard library to encourage adoption.
- **Progression Strategy:** CLI tools → Background Services → Basic GUI → Rich GUI → High-Performance Compute/Graphics.
- **Risks:** Slow initial adoption; lack of native applications.

## 2. Windows Compatibility
**Objective:** Run Windows binaries without recompilation, focusing ultimately on gaming and heavy productivity software.
- **Requirements:** Win32 API translation, PE execution, DirectX to Vulkan mapping, Windows filesystem semantics (case-insensitivity, paths).
- **Technical Approach:** Clean-room implementation of a user-space Win32 translation layer (similar to Wine, but native to Catalyst RPC). Graphics translation via a DXVK-style Vulkan wrapper. Enforcement of Catalyst Core Purity: NO kernel-level anti-cheat hacks; all emulation must occur safely in user space.
- **Legal Considerations:** Must be a strict clean-room reverse engineering effort. No use of leaked Microsoft source code.
- **Progression Strategy:**
  1. Basic headless Win32 executables.
  2. Complex GUI applications.
  3. .NET framework support.
  4. Office-class productivity suites.
  5. Steam client.
  6. High-end PC games.
- **Steam/Gaming Considerations:**
  - **Steam Client:** Requires complex IPC, networking, and filesystem hooks.
  - **DirectX Translation:** Leverage Vulkan heavily. Real-time shader compilation overhead must be minimized.
  - **Anti-Cheat:** Requires careful emulation of Windows kernel semantics in a secure user-space sandbox. The purity principle strictly forbids polluting Catalyst Kernel for anti-cheat requirements.
  - **DRM Considerations:** Support for standard cryptographic APIs and trusted execution environments.
  - **Controller APIs:** Translation of XInput/DirectInput to native Catalyst input server.
- **Risks:** Massive API surface; undocumented Win32 behaviors; anti-cheat kernel-level requirements fundamentally conflict with microkernel security and the purity principle.

## 3. Linux Compatibility
**Objective:** Run standard Linux ELF binaries to bootstrap development tools and server applications.
- **Requirements:** ELF execution, Linux syscall translation.
- **Technical Approach:** A user-space `linuxt` server that catches ELF syscall exceptions or traps and translates them into Catalyst IPC messages. Following the purity principle, Linux-specific primitives (like io_uring or epoll) must be built on top of generic Catalyst async IPC rather than directly implemented in the kernel.
- **Legal Considerations:** Must not link against GPL kernel code; translation layer must be independent.
- **Progression Strategy:** Bash/Coreutils → Compilers (GCC/LLVM) → Networking (Nginx/Redis) → X11/Wayland translation.
- **Risks:** Divergent filesystem semantics; epoll/io_uring translation performance without kernel support.

## 4. Android Compatibility
**Objective:** Run Android APKs to provide a massive mobile application ecosystem.
- **Requirements:** APK parsing, ART/Dalvik runtime execution, Android framework emulation.
- **Technical Approach:** Run a lightweight user-space Android container or integrate the Android Runtime (ART) directly into a Catalyst compatibility layer. Following Catalyst Core Purity, binder and ashmem must be polyfilled in user-space using standard Catalyst IPC and shared memory semantics, not added to the kernel.
- **Legal Considerations:** Google Play Services compatibility is legally and technically gated. Must rely on AOSP and MicroG or similar open alternatives.
- **Progression Strategy:** Headless Java execution → Basic AOSP apps → Complex NDK apps/games.
- **Risks:** Heavy reliance on Linux-specific kernel features requiring careful polyfilling in Catalyst user-space.

## 5. Web Compatibility
**Objective:** First-class support for Web Applications (PWA).
- **Requirements:** Fast JavaScript engine, WebGL/WebGPU support, system integration (notifications, offline storage).
- **Technical Approach:** Port an existing engine (Servo or Chromium/Blink). Expose Catalyst APIs via secure Web APIs. All web rendering isolation happens via native Catalyst capability boundaries.
- **Legal Considerations:** Standard open-source browser engine licensing.
- **Progression Strategy:** Basic HTML/CSS → JS Runtime → Full DOM → WebGPU/WASM → Deep OS integration (PWA).
- **Risks:** Massive codebase porting effort.

## 6. macOS Compatibility
**Objective:** Not explicitly promised. Focus on portable standards.
- **Requirements:** POSIX compliance where sensible, without compromising Catalyst's design or purity principle.
- **Technical Approach:** Rely on Linux compatibility layer or native recompilation for open-source macOS tools. No Mach-O or Cocoa translation layer planned.
- **Legal Considerations:** Apple's EULAs strictly forbid macOS execution on non-Apple hardware.
- **Progression Strategy:** N/A.
- **Risks:** Alienating macOS developers (mitigated by Linux/POSIX compatibility).
