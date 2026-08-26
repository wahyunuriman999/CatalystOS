// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

pub mod capability;

use alloc::collections::VecDeque;
use spin::Mutex;
pub use capability::{CapabilityHandle, CapabilityTable, CapError, CAP_SEND, CAP_RECEIVE, CAP_CALL};

pub const MAX_MESSAGE_SIZE: usize = 256;
pub const MAX_QUEUE_DEPTH: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointId {
    pub index: u32,
    pub generation: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub sender: u64, // Process ID of sender
    pub length: usize,
    pub data: [u8; MAX_MESSAGE_SIZE],
}

impl Message {
    pub fn new(sender: u64, payload: &[u8]) -> Option<Self> {
        if payload.len() > MAX_MESSAGE_SIZE {
            return None;
        }
        let mut data = [0u8; MAX_MESSAGE_SIZE];
        data[..payload.len()].copy_from_slice(payload);
        Some(Message {
            sender,
            length: payload.len(),
            data,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointState {
    Free,
    Active,
    Closed,
}

pub struct Endpoint {
    pub id: EndpointId,
    pub owner: u64,
    pub queue: VecDeque<Message>,
    pub state: EndpointState,
}

impl Endpoint {
    pub fn new(index: u32, generation: u32) -> Self {
        Endpoint {
            id: EndpointId { index, generation },
            owner: 0,
            queue: VecDeque::with_capacity(MAX_QUEUE_DEPTH),
            state: EndpointState::Free,
        }
    }
}

pub struct EndpointRegistry {
    endpoints: alloc::vec::Vec<Endpoint>,
    next_index: u32,
}

impl EndpointRegistry {
    pub const fn new() -> Self {
        EndpointRegistry {
            endpoints: alloc::vec::Vec::new(),
            next_index: 0,
        }
    }

    pub fn create_endpoint(&mut self, owner_pid: u64) -> Result<EndpointId, &'static str> {
        // Try to find a free slot first
        for ep in self.endpoints.iter_mut() {
            if ep.state == EndpointState::Free || ep.state == EndpointState::Closed {
                ep.id.generation = ep.id.generation.wrapping_add(1);
                ep.owner = owner_pid;
                ep.queue.clear();
                ep.state = EndpointState::Active;
                return Ok(ep.id);
            }
        }

        // Expand if no free slot
        let index = self.endpoints.len() as u32;
        let generation = 1;
        let mut ep = Endpoint::new(index, generation);
        ep.owner = owner_pid;
        ep.state = EndpointState::Active;
        self.endpoints.push(ep);
        Ok(EndpointId { index, generation })
    }

    pub fn destroy_endpoint(&mut self, id: EndpointId) -> Result<(), &'static str> {
        if let Some(ep) = self.endpoints.get_mut(id.index as usize) {
            if ep.id.generation != id.generation {
                return Err("Dangling or mismatched endpoint generation");
            }
            if ep.state == EndpointState::Active {
                ep.state = EndpointState::Closed;
                ep.queue.clear();
                return Ok(());
            }
            return Err("Endpoint not active");
        }
        Err("Endpoint out of bounds")
    }

    // Tick 10: Non-blocking send/receive core primitive (will be expanded in Tick 12)
    pub fn send(&mut self, id: EndpointId, msg: Message) -> Result<(), &'static str> {
        if let Some(ep) = self.endpoints.get_mut(id.index as usize) {
            if ep.id.generation != id.generation {
                return Err("Dangling or mismatched endpoint generation");
            }
            if ep.state != EndpointState::Active {
                return Err("Endpoint is closed");
            }
            if ep.queue.len() >= MAX_QUEUE_DEPTH {
                return Err("Queue full");
            }
            ep.queue.push_back(msg);
            Ok(())
        } else {
            Err("Endpoint out of bounds")
        }
    }

    pub fn receive(&mut self, id: EndpointId) -> Result<Message, &'static str> {
        if let Some(ep) = self.endpoints.get_mut(id.index as usize) {
            if ep.id.generation != id.generation {
                return Err("Dangling or mismatched endpoint generation");
            }
            if ep.state != EndpointState::Active {
                return Err("Endpoint is closed");
            }
            if let Some(msg) = ep.queue.pop_front() {
                Ok(msg)
            } else {
                Err("Queue empty")
            }
        } else {
            Err("Endpoint out of bounds")
        }
    }
}

pub static IPC_REGISTRY: Mutex<EndpointRegistry> = Mutex::new(EndpointRegistry::new());

// ─── Capability-Validated Syscall Boundary ───────────────────────────────────
// These are the ONLY entry points for IPC operations from userspace.
// Raw EndpointId is NEVER accepted from userspace.
// Validation order (C4, C5, C12): Handle → Generation → Rights → Endpoint → IPC Core

/// Capability-validated send.
/// Userspace presents CapabilityHandle; kernel resolves to EndpointId internally.
pub fn cap_send(
    table: &CapabilityTable,
    handle: CapabilityHandle,
    payload: &[u8],
    sender_pid: u64,
) -> Result<(), CapError> {
    // Guard 1–4: validate handle, generation, rights (C4, C5, C12)
    let endpoint_id = table.validate(handle, CAP_SEND)?;

    // Guard 5: Construct bounded message (C2 — enforced in Message::new)
    let msg = Message::new(sender_pid, payload)
        .ok_or(CapError::IpcError)?;

    // Guard 6: Send through IPC core (C1, C3 — endpoint state + queue depth)
    IPC_REGISTRY.lock().send(endpoint_id, msg)
        .map_err(|_| CapError::IpcError)
}

/// Capability-validated receive.
pub fn cap_receive(
    table: &CapabilityTable,
    handle: CapabilityHandle,
) -> Result<Message, CapError> {
    // Guard 1–4: validate handle, generation, rights (C4, C5, C12)
    let endpoint_id = table.validate(handle, CAP_RECEIVE)?;

    // Receive through IPC core
    IPC_REGISTRY.lock().receive(endpoint_id)
        .map_err(|_| CapError::IpcError)
}
