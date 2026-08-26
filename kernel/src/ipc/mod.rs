use core::sync::atomic::{AtomicU64, Ordering};

pub struct Capability {
    pub object_id: u64,
    pub rights: u32,
}

pub fn send_ipc(target: &Capability, msg: &[u8]) -> Result<(), &'static str> {
    if target.rights & 1 == 0 {
        return Err("No write rights");
    }
    crate::kprintln!("[IPC] Sent {} bytes to object {}", msg.len(), target.object_id);
    Ok(())
}
