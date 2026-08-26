# CATALYSTOS — FORENSIC BASELINE AUDIT

**Audit Date:** 2026-08-27
**Target:** CatalystOS Productization Baseline (Developer Preview 1 Readiness)

---

## A. ACTUAL REPOSITORY STRUCTURE
The workspace consists of a single Cargo workspace containing:
1. `kernel/`: The CatalystOS microkernel (x86_64 `no_std` binary, bootloader_api v0.11).
   - `src/arch/`: Hardware initialization (GDT, IDT, PIC, Syscall MSRs, PS/2 Mouse).
   - `src/console/`: Serial logging (0x3F8) and VGA text/framebuffer console.
   - `src/memory/`: Physical `BitmapFrameAllocator`, Kernel Heap (`BumpAllocator`), `AddressSpace` page mapping, User memory validation (`copy_from_user`, `copy_to_user`).
   - `src/task/`: Preemptive scheduler (`SCHEDULER`), `Process`, `Task`, `context_switch` (`naked_asm`), and `ELF64` loader.
   - `src/ipc/`: Generational `EndpointId`, `EndpointRegistry`, `CapabilityTable`, `CapabilityHandle`, blocking receive, atomic wakeup, RPC `cap_call` / `cap_reply`.
   - `src/storage/`: `VFS` Inode/VNode abstraction, `RamFS`, `RamDisk`, `Package` (`CPKG`), `UpdateDescriptor` (A/B slot rollback).
   - `src/drivers/`: PCI discovery, VirtIO block, VirtIO net, Intel HDA audio stubs.
   - `src/graphics/`: Canvas, Color, Compositor, Desktop windowing system.
   - `src/net/`: EthernetHeader, Ipv4Header, UdpHeader, checksum calculation.
   - `src/init/`: `ServiceManager` with crash detection and auto-restart.
   - `src/security/`: `ProcessQuota`, $W\oplus X$ enforcement, canonical address validation, `Watchdog`.
   - `src/compat/`: PE Loader, Win32 translation layer stubs.
   - `src/test_harness.rs`: Integrated kernel test runner (33 tests).
2. `userland/libcatalyst/`: Userspace standard library offering native syscall wrappers (`open`, `read`, `write`, `close`, `exit`, `yield_now`, `getpid`) and formatting macros (`println!`).
3. `userland/hello/`: Native userland test binary compiled for `x86_64-catalyst-user`.
4. `tools/mkimage/`: Bootable disk image builder wrapping `bootloader::DiskImageBuilder`.
5. `docs/`: Architecture specifications, decision matrices, audit reports.

---

## B. ACTUAL IMPLEMENTED SUBSYSTEMS
- **Privilege Separation:** Ring 0 (Kernel) and Ring 3 (User) with dedicated TSS stack (`RSP0`) and MSR-based `syscall`/`sysretq` dispatch.
- **Memory Management:** 4-level x86_64 paging with kernel identity/offset mapping, per-process PML4 address spaces, user pointer range validation ($< 0x0000\_7FFF\_FFFF\_FFFF$).
- **Multitasking:** Preemptive round-robin scheduler with `TaskState::Ready`, `Running`, `Blocked(BlockReason)`, `Dead` and deferred task reaping.
- **Capability-Based IPC:** Bounded messages (256 B), bounded queues (64), generational endpoints, opaque capability handles, explicit rights (`SEND`, `RECEIVE`, `CALL`), blocking receive, atomic wakeups.
- **Virtual File System (VFS):** Hierarchical Inode/VNode architecture with `RamFS`, `/bin`, `/dev`, `/etc`, `/home`, `/tmp`, `/var`, `/sys`, `/proc`, and per-process `FileDescriptorTable`.
- **System Call ABI:** Fast `syscall`/`sysretq` dispatch table with copy-in/copy-out memory validation.
- **Package & Recovery:** Catalyst Package format (`CPKG`), A/B system slot updates with automatic 3-boot failure rollback.

---

## C. CLAIMED VS. VERIFIED FEATURES
| Feature | Claimed Status | Verified Ground Truth |
| :--- | :---: | :--- |
| Ring 3 Execution | Implemented | Verified (Runs user functions via `enter_usermode` & Syscalls) |
| Capability Enforcement | Implemented | Verified (Tests J, K, L, M assert rejection of forged/stale/cross-process handles) |
| Blocking IPC & Wakeup | Implemented | Verified (Tests N, O, P, Q assert atomic wait queues & wakeups) |
| RPC Call / Reply | Implemented | Verified (Test S asserts client unblocking on server reply) |
| VFS File R/W | Implemented | Verified (Tests W, X assert Inode create, read, write, unlink) |
| VirtIO Hardware Drivers | Implemented | Foundation (PCI probe works; interrupt-driven DMA ring buffer pending) |
| Network Protocols | Implemented | Foundation (Header serialization/parsing verified; packet transmission on PCI net pending) |
| Desktop Shell | Implemented | Foundation (Window compositor renders to linear framebuffer; userspace client protocol pending) |
| Win32 Compatibility | Stubbed | Foundation (PE loader parses headers and inserts syscall stubs) |

---

## D. MISSING FEATURES FOR DEVELOPER PREVIEW 1
1. **Real VirtIO Block Persistent Storage:** Replacing memory-backed root FS with live VirtIO block disk mounting.
2. **Userspace Init Binary Loading:** Booting directly into `/bin/init` from disk instead of hardcoded kernel monitor thread.
3. **Userspace Terminal Shell (`sh`):** Interactive prompt reading keyboard scancodes and launching ELF binaries from `/bin/`.
4. **Independent CI Test Oracle:** Decoupling CI assertion from kernel-printed strings using machine-readable QEMU exit codes and guest-host communication.

---

## E. DEAD CODE & COMPILER WARNINGS
- `kernel/src/main.rs`: Unused `desktop_task` stub.
- `kernel/src/compat/pe_loader.rs`: Redundant nested `unsafe` blocks.
- `kernel/src/drivers/pci.rs`: Unused constant vendor IDs (`VENDOR_VIRTIO`, `VENDOR_INTEL`, `VENDOR_AMD`).
- `kernel/src/console/mod.rs`: Redundant null pointer check on array reference.

---

## F. TODO & FIXME AUDIT
- `kernel/src/task/elf.rs`: Program header loading currently performs memory mapping; physical page copy from VFS files needs direct mapper attachment.
- `kernel/src/compat/win32.rs`: Win32 console and process APIs are stubs mapping to kernel console output.

---

## G. UNSAFE BLOCKS AUDIT
- **Total Unsafe Blocks in Kernel:** 48 occurrences.
- **Critical Unsafe Operations:**
  1. `context_switch` (`kernel/src/task/context.rs`): Assembly register pushing/popping and stack pointer switching.
  2. `syscall_entry` (`kernel/src/arch/syscall.rs`): Naked assembly handling `swapgs` and `sysretq`.
  3. `enter_usermode` (`kernel/src/task/process.rs`): Assembly pushing Ring 3 IRET frame.
  4. `active_level_4_table` (`kernel/src/memory/mod.rs`): Direct pointer dereference of CR3 physical offset.
- **Safety Assessment:** All unsafe blocks are confined to hardware boundaries (paging, CPU registers, port I/O, naked context switches) and protected by safe wrapper traits (`AddressSpace`, `FrameAllocator`, `VNode`).

---

## H. ARCHITECTURE VIOLATIONS CHECK
- **No Implicit Shared Memory:** Verified. All IPC transfers use bounded message copying or explicit capability grants.
- **Microkernel Boundary:** Verified. Subsystems communicate via message structures and VFS interfaces rather than direct global state mutation.

---

## I. FAKE / STUB IMPLEMENTATIONS
- `drivers/virtio_block.rs` & `drivers/virtio_net.rs`: Probes PCI and reads BAR0/MAC address, but high-throughput DMA rings are currently abstracted via `RamDisk` and packet parser suites.
- `compat/win32.rs`: Basic Win32 console output hooks present for emulation testing.

---

## J. TEST HARNESS WEAKNESSES
- Currently, `test_harness.rs` runs from a spawned kernel task (`monitor_thread`).
- If an assertion fails, the kernel task panics and serial prints the panic message, but QEMU does not automatically terminate with an exit code unless debug exit device (`isa-debug-exit`) is invoked.
- **Required Fix:** Integrate `isa-debug-exit` (Port 0xF4) so test failures cause QEMU process exit code `1` (failure) and success causes exit code `0` (success).

---

## K. CI & BUILD WEAKNESSES
- Build script `tools/build-and-run.ps1` exists for local testing but lacks a unified, deterministic multiplatform `scripts/build.sh` and `scripts/build.ps1`.
- Need strict CI validation without `|| true` suppressions.

---

## L. BOOT PATH
`bootloader` (UEFI/BIOS)
  $\rightarrow$ `kernel_main` (`kernel/src/main.rs`)
  $\rightarrow$ `console::init()` (Serial 0x3F8 + VGA Framebuffer)
  $\rightarrow$ `memory::init(boot_info)` (OffsetPageTable, BitmapFrameAllocator, Heap)
  $\rightarrow$ `arch::init()` (GDT, TSS, IDT, PICs, Syscall MSRs)
  $\rightarrow$ `storage::init()` (VFS Root, RamFS, System directories)
  $\rightarrow$ `net::init_net()` (Network subsystem, loopback)
  $\rightarrow$ `init::start_init_process()` (ServiceManager, core daemons)
  $\rightarrow$ `security::init_security()` (Quotas, W^X, Watchdog)
  $\rightarrow$ `test_harness::run_all_tests()` (Spawns monitor thread)
  $\rightarrow$ Preemptive timer interrupt scheduler loop (`x86_64::instructions::hlt()`).

---

## M. USERSPACE BOOT PATH
`Process::new(pid)`
  $\rightarrow$ Allocates PML4 `AddressSpace` with copied kernel upper half ($0xFFFF\_8000\_0000\_0000..$)
  $\rightarrow$ `load_elf_into_address_space` maps `.text`, `.rodata`, `.data`, `.bss`, and User Stack
  $\rightarrow$ `enter_usermode` switches CPU to Ring 3 ($CS=0x23, SS=0x1B$)
  $\rightarrow$ Program calls `syscall` $\rightarrow$ Kernel `syscall_entry` $\rightarrow$ `sysretq` back to Ring 3.

---

## N. DRIVER INITIALIZATION PATH
`drivers::init()`
  $\rightarrow$ `pci::enumerate()` scans Bus 0..255, Device 0..31, Function 0..7
  $\rightarrow$ Detects VirtIO Net, VirtIO Block, Intel HDA Audio
  $\rightarrow$ Binds device BARs and registers interrupt handlers.

---

## O. SERVICE STARTUP PATH
`init_services()`
  $\rightarrow$ Registers `vfsd`, `netd`, `displayd`, `inputd`, `logd` in `SERVICE_MANAGER`
  $\rightarrow$ Assigns IPC endpoints and auto-restart monitors.

---

## P. PACKAGE / UPDATE PATH
`install_package(&cpkg_bytes)`
  $\rightarrow$ `PackageHeader::parse` verifies magic (`CPKG`), length, and checksum
  $\rightarrow$ Atomically writes binary payload into `/bin/<name>` via VFS
  $\rightarrow$ `UpdateDescriptor` coordinates A/B slot transitions with 3-boot attempt watchdog rollback.
