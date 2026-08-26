<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS Filesystem Specification v0.2

This document outlines the filesystem strategy for Catalyst OS. The design follows a strict, incremental implementation path, ensuring correct foundational layers before moving to production-grade complexity. We explicitly do NOT front-load filesystem work.

## 1. Filesystem Strategy Order (Approved Progression)
1. **Boot → Kernel → Memory → Scheduler → Process → Syscall**: Foundational OS components must be solid before any complex storage logic is introduced.
2. **CatRAM**: A simple, temporary, in-memory filesystem for early testing and user-space bootstrapping. (APPROVED)
3. **Basic Storage Abstraction**: Block device drivers and abstract I/O traits (the block layer).
4. **VFS (Virtual File System) Layer**: Mount points, file descriptors, paths, and generic node representation in user-space.
5. **CatFS v0**: A simple, correct, on-disk persistent filesystem for basic storage.
6. **CatFS Production**: A modern filesystem with CoW (Copy-on-Write), checksums, and snapshots. To be implemented much later.

## 2. Phase 2: CatRAM (In-Memory FS)
**Decision:** Implement `CatRAM` as the first step for filesystem testing.
- **Requirements:** Must hold initial user-space servers (init, VFS, shell). Read/write capability for scratch files. Minimal code footprint. No persistence.
- **Rationale:** The absolute minimum viable product to bootstrap user-space without dealing with hardware drivers or disk corruption bugs during early kernel dev. It isolates VFS development from block device driver development.

## 3. Phase 3: Storage Abstraction Layer Design
**Decision:** Abstract block devices via asynchronous, fixed-size block traits in Rust.
- **Requirements:** Must support polling (early boot) and interrupts/DMA (production). Must abstract over RAM, AHCI/SATA, and NVMe.
- **Rationale:** A simple async block trait (`AsyncReadBlock`, `AsyncWriteBlock`) provides the right balance, allowing the kernel to scale cleanly from early polling to high-performance DMA.

## 4. Phase 4: VFS Architecture
**Decision:** Component-based VFS living mostly in a user-space server (hybrid microkernel design), communicating with the kernel via fast IPC.
- **Requirements:** Support hierarchical namespace, multiple mount points, permission checks, and generic file descriptors.
- **Rationale:** Safety and modularity. IPC latency is mitigated by aggressive batching and shared memory windows for large reads/writes. The kernel only manages IPC handles and memory maps.

## 5. Phase 5: CatFS v0
**Decision:** A basic, custom persistent filesystem.
- **Requirements:** Superblock, inodes, directory entries, basic crash consistency.
- **Rationale:** Introduces persistent on-disk storage using the new block layer and VFS layer. Focus is purely on correctness, not performance.

## 6. Phase 6: CatFS Production Requirements
**Decision:** `CatFS`, a modern, CoW (Copy-on-Write) B-tree based filesystem optimized for NVMe.
- **Requirements:** 
  - Crash consistency (atomic updates).
  - Data and metadata checksums (prevent silent rot).
  - Fast snapshots.
  - Transparent compression.
  - SSD/NVMe optimization (concurrent queues, lock-free parallel reads).
- **Rationale:** NVMe drives handle random I/O extremely well, mitigating CoW fragmentation penalties. The benefits of easy snapshots (for system rollback/updates) and data integrity (checksums) align with Catalyst OS's goals of reliability and developer experience.

### Catalyst-Specific Design Decisions
- **Decision:** Asynchronous I/O Only. No blocking file APIs in the VFS layer; all operations use an io_uring-like ring buffer interface over IPC.
- **Decision:** Unified Page Cache. The filesystem does not maintain its own cache; it heavily relies on the kernel's virtual memory manager (VMM) page cache to avoid double-caching.
