<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# CATALYST OS SECURITY MODEL v0.1

## 1. Threat Model
Catalyst OS operates under a Zero Trust assumption. We are protecting the system and user data against:
- **Malware & Untrusted Applications:** Malicious or poorly written code attempting unauthorized access, data destruction, or resource abuse.
- **Privilege Escalation:** Exploits attempting to gain elevated privileges (root/admin) from unprivileged contexts.
- **Data Theft & Exfiltration:** Unauthorized reading and transmission of user data, secrets, or system state.
- **Supply Chain Attacks:** Compromised third-party packages, libraries, or system components.
- **Network-Borne Threats:** Remote code execution, man-in-the-middle attacks, and unauthorized network probing.
- **Physical Access:** Attackers with physical access attempting cold boot attacks, evil maid attacks, or direct storage reads.

## 2. Capability-Based Security
**Evaluation:**
- *ACL-Based (Discretionary/Mandatory):* Complex to manage, subject to ambient authority problems (e.g., confused deputy), difficult to track precisely what an app can do.
- *Capability-Based:* Grants fine-grained authority via unforgeable tokens (object references). Eliminates ambient authority.
- *Hybrid:* Combines broad ACLs with capabilities. Often inherits the worst of both.

**Decision & Rationale:**
Catalyst OS uses a **pure capability-based security model** for inter-component communication and resource access. Every system resource (file, socket, hardware) is represented as a capability. A process can only interact with a resource if it holds the corresponding capability. This inherently prevents the "confused deputy" problem and aligns with our Rust-based microkernel architecture, where capabilities are passed as secure IPC messages.

## 3. Process Isolation
Processes in Catalyst OS are heavily isolated by default:
- **Address Space Isolation:** Every process runs in its own distinct virtual address space enforced by hardware MMU.
- **Microkernel Syscall Filtering:** The kernel exposes a minimal attack surface. Most operations are handled via IPC to user-space servers, rather than direct syscalls.
- **Resource Limits:** Strict quotas on CPU time, memory allocation, and I/O bandwidth per process/cgroup to prevent denial-of-service (DoS) attacks.
- **No Global Namespace:** There is no shared global filesystem or registry. Processes only see the resources (files/directories) explicitly passed to them via capabilities.

## 4. Application Sandbox
The application lifecycle is strictly controlled:
1. **Install:** Packages are verified against their cryptographic signatures.
2. **Verify:** Static analysis and manifest validation.
3. **Sandbox:** Apps are installed into isolated, immutable directories. They cannot write to their own executable directory.
4. **Permissions:** Requested capabilities are presented to the user.
5. **Run:** The app is spawned with only its requested and approved capabilities. It has no ambient access to the network or user filesystem.
6. **Update:** Cryptographically verified differential updates.
7. **Uninstall:** Completely removes all app data and capabilities, leaving no orphaned files or registry entries.

## 5. Permission Model
Catalyst OS employs a mobile-style, dynamic permission model.
- **Granular Rights:** Microphone, camera, precise/coarse location, clipboard access, specific files/folders, network interfaces, screen capture, input event capture.
- **Time-Bound Grants:**
  - *Allow once:* Permission expires when the app closes or the action completes.
  - *Allow while running:* Permission is granted only when the app is in the foreground.
  - *Permanent:* Explicit user trust required.
  - *Deny / Revoke:* Permissions can be revoked at any time via the centralized Privacy Center.

## 6. Secure Boot
Catalyst OS fully integrates with **UEFI Secure Boot**.
- The bootloader is signed with a Catalyst OS specific key (and Microsoft 3rd Party key for compatibility).
- The bootloader verifies the Rust microkernel image and initial ramdisk before execution.
- Any unauthorized modification to the boot chain results in a fallback to recovery mode or halt.
- Measured Boot (TPM integration) allows attestation of the boot sequence.

## 7. Signed Packages
- All first-party components and third-party applications must be cryptographically signed.
- **Trust Chain:** Root CA -> Developer Certificates -> Package Signature.
- The package manager rejects any package with an invalid, expired, or untrusted signature.
- Revocation lists (CRLs) are maintained to invalidate compromised developer keys instantly.

## 8. IPC Security
Inter-Process Communication (IPC) is the backbone of the microkernel.
- All IPC is mediated by the kernel.
- **Authorization:** Only processes holding a valid capability for a specific IPC endpoint can send messages to it.
- **Immutability:** IPC messages are copied between address spaces; they cannot be tampered with mid-transit.
- **Endpoint Types:** Typed capabilities prevent sending inappropriate data structures to endpoints expecting different formats.

## 9. Exploit Mitigation
Catalyst OS implements modern exploit mitigations at all levels:
- **Memory Safety:** The kernel and core servers are written in Rust, eliminating entire classes of memory corruption bugs (use-after-free, buffer overflows).
- **ASLR & KASLR:** Address Space Layout Randomization (and Kernel ASLR) forces random memory placement for all binaries and libraries.
- **DEP/NX:** Data Execution Prevention / No-eXecute bit ensures writable memory is not executable (W^X).
- **Stack Canaries:** Compiler-inserted guards against stack buffer overflows.
- **CFI / Shadow Stacks:** Control Flow Integrity and hardware shadow stacks (Intel CET) prevent ROP/JOP attacks.

## 10. Secret Management
- **Key Storage:** Cryptographic keys are stored in a secure enclave or backed by the hardware TPM.
- **Credential Isolation:** The Credential Manager runs as an isolated server. Applications cannot read credentials directly; they request operations (like signing or authentication) via capability-restricted IPC.
- **User Passwords:** Never stored in plaintext; hashed using memory-hard functions (e.g., Argon2id).

## 11. AI Agent Security
In Catalyst OS, the AI is treated as a highly capable but fundamentally unprivileged user-space agent.
- **Not Root:** The AI agent does not have root or administrative privileges by default.
- **Permission-Controlled Actions:** Any action the AI attempts to perform (modifying system settings, accessing files, making network requests) is subject to the same capability checks as any other application.
- **Audit Logging:** All actions taken by the AI agent are securely logged for user review.
- **Explicit Consent:** High-risk actions (deleting files, installing software, sending data externally) require explicit user confirmation via a secure, un-spoofable UI prompt.
