<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS Filesystem Specification v0.1

This document outlines the filesystem strategy for Catalyst OS. The design follows a strict, incremental implementation path, ensuring correct foundational layers before moving to production-grade complexity.

## 1. Filesystem Strategy Order
1. **Boot → Kernel → Memory → Scheduler → Process → Syscall**: Foundational OS components must be solid before complex storage logic is introduced.
2. **Basic Storage Abstraction**: Block device drivers and abstract I/O traits.
3. **VFS (Virtual File System) layer**: Mount points, file descriptors, paths, and generic node representation.
4. **Filesystem v0**: In-memory or very simple RAM-disk (initrd/tmpfs equivalent) for early user-space and testing.
5. **Production Filesystem**: Persistent, crash-safe on-disk filesystem (later milestone).

## 2. Storage Abstraction Layer Design

**Decision:** Abstract block devices via asynchronous, fixed-size block traits in Rust.
- **Requirements:** Must support polling (early boot) and interrupts/DMA (production). Must abstract over RAM, AHCI/SATA, and NVMe.
- **Alternatives:** Synchronous blocking I/O (too slow for user-space), complex bio-structs like Linux (too heavy for early phase).
- **Trade-offs:** Asynchronous traits in Rust require an executor and pin/waker complexity.
- **Rationale:** A simple async block trait (`AsyncReadBlock`, `AsyncWriteBlock`) provides the right balance, allowing the kernel to scale cleanly from early polling to high-performance DMA.

## 3. VFS Architecture

**Decision:** Component-based VFS living mostly in a user-space server (hybrid microkernel design), communicating with the kernel via fast IPC.
- **Requirements:** Support hierarchical namespace, multiple mount points, permission checks, and generic file descriptors.
- **Alternatives:** Monolithic in-kernel VFS (Linux-style, high performance but poor isolation). Pure microkernel (every FS is a separate server, high IPC overhead).
- **Trade-offs:** Moving VFS to user-space incurs IPC overhead per syscall.
- **Rationale:** Safety and modularity. IPC latency is mitigated by aggressive batching and shared memory windows for large reads/writes. The kernel only manages IPC handles and memory maps.

## 4. Filesystem v0 Requirements

**Decision:** Implement `CatRAM`, a simple, temporary, in-memory filesystem.
- **Requirements:** Must hold initial user-space servers (init, VFS, shell). Read/write capability for scratch files. Minimal code footprint. No persistence.
- **Alternatives:** Port an existing simple FS like FAT32 (too much legacy baggage, unnecessary disk I/O for early boot).
- **Trade-offs:** RAM is volatile. It does not test block device drivers.
- **Rationale:** The absolute minimum viable product to bootstrap user-space without dealing with hardware drivers or disk corruption bugs during early kernel dev.

## 5. Evaluation of Existing Filesystems

- **ext4:** 
  - *Learnings:* Excellent performance, proven journal. 
  - *Rejected:* Legacy structures, not designed for modern NVMe paradigms (extent trees are bolted on).
- **Btrfs:** 
  - *Learnings:* CoW, snapshots, checksums. 
  - *Rejected:* Extreme complexity, historical stability issues with ENOSPC and RAID.
- **ZFS:** 
  - *Learnings:* Flawless data integrity, ARC caching, logical volume management integration. 
  - *Rejected:* Massive memory footprint, not suitable for a microkernel base system.
- **F2FS:** 
  - *Learnings:* Log-structured design optimized for flash memory. 
  - *Rejected:* Too specialized for NAND characteristics that modern NVMe controllers already abstract.
- **NTFS:** 
  - *Learnings:* Rich metadata, ACLs, Alternate Data Streams. 
  - *Rejected:* Complex and proprietary structure, sluggish metadata performance.

## 6. Production FS Requirements & Catalyst Specification

**Decision:** `CatFS`, a modern, CoW (Copy-on-Write) B-tree based filesystem optimized for NVMe.
- **Requirements:** 
  - Crash consistency (atomic updates).
  - Data and metadata checksums (prevent silent rot).
  - Fast snapshots.
  - Transparent compression.
  - SSD/NVMe optimization (concurrent queues, lock-free parallel reads).
- **Alternatives:** Journaled filesystem (ext4 style) or Log-structured filesystem.
- **Trade-offs:** CoW introduces fragmentation over time, requiring background defragmentation. However, it naturally provides snapshots and atomic updates without journaling overhead.
- **Rationale:** NVMe drives handle random I/O extremely well, mitigating CoW fragmentation penalties. The benefits of easy snapshots (for system rollback/updates) and data integrity (checksums) align with Catalyst OS's goals of reliability and developer experience.

### Catalyst-Specific Design Decisions
- **Decision:** Asynchronous I/O Only. No blocking file APIs in the VFS layer; all operations use an io_uring-like ring buffer interface over IPC.
  - *Rationale:* Maximizes NVMe concurrency and fits the Rust async paradigm natively.
- **Decision:** Unified Page Cache. The filesystem does not maintain its own cache; it heavily relies on the kernel's virtual memory manager (VMM) page cache to avoid double-caching.
  - *Rationale:* Memory efficiency (aligns with the <500MB idle footprint goal).
