<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# CATALYST OS — CORE PRIMITIVES CONTRACT SPECIFICATION v1.0

## 1. Architectural Philosophy: The Lean Microkernel Invariant

Catalyst OS strictly separates **Mechanism (Microkernel)** from **Policy & Experience (Userspace Platform)**.

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                           CATALYST EXPERIENCE                           │
│     Spatial Workspaces • Living Links • Three-Tier Snapshots • Objects   │
├─────────────────────────────────────────────────────────────────────────┤
│                        USERSPACE PLATFORM DAEMONS                       │
│   objectd • workspaced • stated • displayd • inputd • sessiond • cpkg   │
├─────────────────────────────────────────────────────────────────────────┤
│                           MICROKERNEL CORE                              │
│       Process • AddressSpace • Capability • IPC • Scheduler • Traps     │
└─────────────────────────────────────────────────────────────────────────┘
```

The Microkernel **NEVER** manages complex business logic, UI layouts, or application-level object schemas. It provides non-forgeable capabilities, fast IPC, address space isolation, and preemptive execution. All high-level paradigms operate in Ring 3 userspace daemons.

---

## 2. Formal Contract for the 10 Core Primitives

### Primitive 1: Object (`CatalystObject`)
- **Domain:** Userspace (`objectd`) backed by persistent VFS.
- **Identity:** Globally unique 64-bit integer (`ObjectId`).
- **Ownership:** Bound to `OwnerUid` / `OwnerPid` with explicit capability access rights.
- **Type Classification:** `Document`, `Spreadsheet`, `Media`, `Code`, `SpatialScene`, `Stream`, `Directory`, `GenericBinary`.
- **Lifecycle:** `Allocated` $\rightarrow$ `Active` $\rightarrow$ `Snapshotted` $\rightarrow$ `Archived` $\rightarrow$ `Reclaimed`.
- **Authority:** Zero Ambient Authority. Operations require explicit capability handles (`CAP_READ`, `CAP_WRITE`, `CAP_SHARE`, `CAP_LINK`).
- **Persistence:** Metadata stored in indexed object database; payload stored in persistent block storage.

---

### Primitive 2: Capability (`CapabilityHandle`)
- **Domain:** Microkernel (`kernel/src/ipc/capability.rs`).
- **Contract:** Non-forgeable opaque tuple `(SlotIndex, Generation)`.
- **Right Mask:** Bitwise permissions (`CAP_SEND`, `CAP_RECEIVE`, `CAP_CALL`, `CAP_REPLY`, `CAP_SHM_READ`, `CAP_SHM_WRITE`).
- **Revocation Semantics:** Incrementing generation immediately invalidates all outstanding handles.
- **Cross-Process Protection:** Capabilities cannot be transferred except via explicit IPC delegation with kernel mediation.

---

### Primitive 3: State (`StateDescriptor`)
- **Domain:** Shared between Kernel (Hardware/Process context) and Userspace (`stated`).
- **Contract:** Complete serialized snapshot of memory bounds, register state, spatial coordinates, or object data.
- **Determinism:** Given identical state inputs, execution and rendering produce bit-exact identical outputs.

---

### Primitive 4: Workspace (`LivingWorkspace`)
- **Domain:** Userspace (`workspaced` & `displayd`).
- **Contract:** Boundless spatial coordinates $(x, y, z, \text{scale})$ holding active spatial nodes, surface windows, and bound objects.
- **Boundary & Ownership:** Each workspace is an isolated security and visual domain.
- **Crash Recovery:** If a window crashes, the workspace retains the node placeholder and prompts for instant service/process reboot.

---

### Primitive 5: Relationship (`SemanticLink`)
- **Domain:** Userspace (`objectd`).
- **Types:** `DerivedFrom`, `ReferencedBy`, `ParentOf`, `ChildOf`, `LivingLink`, `TemporalPredecessor`.
- **Zero-Magic Enforcement:** When Object A changes:
  1. A change event is broadcast to the relationship resolver in `objectd`.
  2. Affected dependent objects are identified.
  3. Notifications are dispatched to listening applications.
  4. **NO implicit execution** occurs without explicit user/policy approval.
- **Cycle Prevention:** Directed Acyclic Graph (DAG) validation at relationship registration.

---

### Primitive 6: Snapshot (`Three-Tier Snapshot Architecture`)
- **Domain:** Multi-tier coordinated by `stated`, `objectd`, and `vfsd`.
- **Level 1 (Object Snapshot):** Individual document/file versioning (`data`, `checksum`, `version_number`).
- **Level 2 (Workspace Snapshot):** Spatial canvas layout, open window positions, visual zoom, and active object bindings.
- **Level 3 (System Recovery Snapshot):** Root filesystem state, installed package manifests, system configuration, and A/B kernel rollback slot.

---

### Primitive 7: Surface (`CatalystSurface`)
- **Domain:** Userspace (`displayd`).
- **Contract:** Shared-memory 32-bit ARGB pixel buffer backed by `CAP_SHM_WRITE`.
- **Damage Protocol:** Applications commit `DamageRect(x, y, w, h)` bounding boxes to minimize compositing overhead.
- **Z-Order Management:** Strict window hierarchy with focus tracking and spatial z-depth sorting.

---

### Primitive 8: Event (`InputEvent` & `SystemEvent`)
- **Domain:** Microkernel capture $\rightarrow$ normalized in `inputd` $\rightarrow$ routed to `displayd`.
- **Contract:** Strongly-typed immutable messages (`KeyDown`, `KeyUp`, `PointerMove`, `ButtonDown`, `ButtonUp`, `SurfaceFocus`, `BubblePrompt`).

---

### Primitive 9: Process (`Task` & `Process`)
- **Domain:** Microkernel (`kernel/src/task/process.rs`).
- **Contract:** Preemptive thread of execution running in Ring 3 with private 4-level page table, quota enforcement, and capability table.
- **Lifecycle:** `Ready` $\rightarrow$ `Running` $\rightarrow$ `Blocked(Reason)` $\rightarrow$ `Dead` $\rightarrow$ `Reaped`.

---

### Primitive 10: Service (`SystemDaemon`)
- **Domain:** Userspace managed by `init` / `SERVICE_MANAGER`.
- **Contract:** Long-running system daemon with supervised lifetime, health heartbeats, crash storm throttling, and auto-restart policy.

---

## 3. Permission Bubbles Security Specification

```text
┌────────────────────────────────────────────────────────┐
│                   PERMISSION BUBBLE                    │
│                                                        │
│  Application: Photos (PID 42)                         │
│  Ambient Authority: ZERO                               │
│                                                        │
│  Resource Requests:                                    │
│   ├── /home/user/Pictures  ──► [ GRANTED / ALLOW ]     │
│   ├── /home/user/Documents ──► [ REJECTED / DENY ]     │
│   ├── Network Socket       ──► [ REJECTED / DENY ]     │
│   └── Camera Device        ──► [ PROMPT USER ]         │
└────────────────────────────────────────────────────────┘
```

1. **Zero Ambient Authority:** Every Ring 3 process starts with an empty capability table.
2. **Interactive Promotion:** Any attempt to access an ungranted resource triggers a synchronous IPC prompt to the userland Security UI.
3. **Instant Revocation:** Permissions can be revoked in real-time by the user, severing IPC and shared-memory capability handles immediately.

---

<!-- End of Specification -->
