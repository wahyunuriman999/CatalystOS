// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use alloc::vec::Vec;
use crate::ipc::EndpointId;

// Rights flags — plain constants, never exposed raw to userspace
pub const CAP_SEND: u8    = 1 << 0;
pub const CAP_RECEIVE: u8 = 1 << 1;
pub const CAP_CALL: u8    = 1 << 2;

/// Opaque handle. This is the ONLY token userspace ever holds.
/// It has no semantic content — it is meaningless outside the owning process's
/// CapabilityTable. Guessing or forging this does NOT grant access (C12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityHandle {
    pub slot: u32,
    pub generation: u32, // Incremented on every slot revoke (C1 generational safety)
}

/// Internal kernel representation. Never leaves kernel memory.
struct CapabilitySlot {
    endpoint: EndpointId, // Full generational EndpointId from Tick 10
    rights: u8,
}

/// Per-process/thread capability table.
/// Lives entirely in kernel memory. Owned by the Process.
/// Userspace never has a reference or pointer into this structure.
pub struct CapabilityTable {
    slots: Vec<Option<CapabilitySlot>>,
    // Slot generation is tracked INDEPENDENTLY from slot content.
    // When a slot is freed (revoked), its generation increments.
    // This ensures old handles pointing to freed+reused slots are rejected (C1, C12).
    slot_generations: Vec<u32>,
    owner_pid: u64,
}

impl CapabilityTable {
    pub fn new(owner_pid: u64) -> Self {
        CapabilityTable {
            slots: Vec::new(),
            slot_generations: Vec::new(),
            owner_pid,
        }
    }

    pub fn owner_pid(&self) -> u64 {
        self.owner_pid
    }

    /// Kernel-internal: Grant access to an endpoint with specified rights.
    /// Returns an opaque CapabilityHandle for the caller to use at syscall boundary.
    pub fn grant(&mut self, endpoint: EndpointId, rights: u8) -> CapabilityHandle {
        // Find a previously freed slot first
        for (idx, slot_opt) in self.slots.iter_mut().enumerate() {
            if slot_opt.is_none() {
                let cur_gen = self.slot_generations[idx]; // Already incremented on last revoke
                *slot_opt = Some(CapabilitySlot { endpoint, rights });
                return CapabilityHandle { slot: idx as u32, generation: cur_gen };
            }
        }
        // No free slot — expand table
        let idx = self.slots.len() as u32;
        self.slots.push(Some(CapabilitySlot { endpoint, rights }));
        self.slot_generations.push(1); // First generation starts at 1
        CapabilityHandle { slot: idx, generation: 1 }
    }

    /// Kernel-internal: Revoke a capability.
    /// After revoke, the slot's generation increments, making any copies of the old
    /// handle permanently invalid (C12 non-forgery + C1 generational safety).
    pub fn revoke(&mut self, handle: CapabilityHandle) -> Result<(), CapError> {
        let cur_gen = self.slot_generations.get(handle.slot as usize)
            .copied()
            .ok_or(CapError::InvalidHandle)?;
        if cur_gen != handle.generation {
            return Err(CapError::StaleHandle);
        }
        if self.slots[handle.slot as usize].is_none() {
            return Err(CapError::InvalidHandle);
        }
        self.slots[handle.slot as usize] = None;
        // Increment generation — old handle copies are now permanently invalid
        self.slot_generations[handle.slot as usize] = cur_gen.wrapping_add(1);
        Ok(())
    }

    /// Validate a CapabilityHandle and check for required rights.
    /// This is the ONLY path from CapabilityHandle -> EndpointId.
    /// All IPC operations MUST pass through here before touching IPC core (C4, C5, C12).
    pub fn validate(&self, handle: CapabilityHandle, required_rights: u8) -> Result<EndpointId, CapError> {
        // Guard 1: Slot in bounds
        let cur_gen = self.slot_generations.get(handle.slot as usize)
            .ok_or(CapError::InvalidHandle)?;
        // Guard 2: Generation must match (rejects forged, stale handles)
        if *cur_gen != handle.generation {
            return Err(CapError::StaleHandle);
        }
        // Guard 3: Slot must be occupied
        let slot = self.slots[handle.slot as usize].as_ref()
            .ok_or(CapError::InvalidHandle)?;
        // Guard 4: Rights must satisfy all required bits (C5)
        if slot.rights & required_rights != required_rights {
            return Err(CapError::InsufficientRights);
        }
        // Only now is EndpointId revealed to the IPC layer
        Ok(slot.endpoint)
    }

    /// Revoke ALL capabilities in this table pointing to a given endpoint.
    /// Used during endpoint destruction to prevent dangling capabilities (C8, C14).
    pub fn revoke_all_for_endpoint(&mut self, endpoint: EndpointId) {
        for (idx, slot_opt) in self.slots.iter_mut().enumerate() {
            let matches = slot_opt.as_ref().map(|s| {
                s.endpoint.index == endpoint.index
                    && s.endpoint.generation == endpoint.generation
            }).unwrap_or(false);
            if matches {
                *slot_opt = None;
                self.slot_generations[idx] = self.slot_generations[idx].wrapping_add(1);
            }
        }
    }
}

/// Structured error type for capability enforcement violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapError {
    /// Handle slot out of bounds or slot is empty — forged/invalid handle (C12)
    InvalidHandle,
    /// Handle generation does not match — stale or recycled handle (C1, C12)
    StaleHandle,
    /// Rights check failed — caller lacks required permission (C5)
    InsufficientRights,
    /// IPC core layer error (queue full, endpoint closed, etc.)
    IpcError,
}

impl CapError {
    pub fn as_str(&self) -> &'static str {
        match self {
            CapError::InvalidHandle      => "CAPABILITY DENIED: Invalid or forged handle",
            CapError::StaleHandle        => "CAPABILITY DENIED: Stale handle (generation mismatch)",
            CapError::InsufficientRights => "CAPABILITY DENIED: Insufficient rights",
            CapError::IpcError           => "IPC: Core layer error",
        }
    }
}
