// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemSlot {
    SlotA,
    SlotB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    Committed,
    PendingValidation,
    RollbackRequired,
}

#[derive(Debug, Clone)]
pub struct UpdateDescriptor {
    pub active_slot: SystemSlot,
    pub target_slot: Option<SystemSlot>,
    pub status: UpdateStatus,
    pub boot_attempts: u32,
    pub max_attempts: u32,
    pub image_checksum: u32,
}

impl UpdateDescriptor {
    pub const fn new() -> Self {
        UpdateDescriptor {
            active_slot: SystemSlot::SlotA,
            target_slot: None,
            status: UpdateStatus::Committed,
            boot_attempts: 0,
            max_attempts: 3,
            image_checksum: 0,
        }
    }

    /// Prepare atomic A/B slot switch.
    pub fn stage_update(&mut self, target: SystemSlot, checksum: u32) {
        self.target_slot = Some(target);
        self.status = UpdateStatus::PendingValidation;
        self.boot_attempts = 0;
        self.image_checksum = checksum;
    }

    /// Called on successful system boot to mark update as committed.
    pub fn commit_successful_boot(&mut self) {
        if let Some(target) = self.target_slot {
            self.active_slot = target;
            self.target_slot = None;
        }
        self.status = UpdateStatus::Committed;
        self.boot_attempts = 0;
    }

    /// Called during bootloader/early init to record a boot attempt.
    pub fn record_boot_attempt(&mut self) -> Result<(), &'static str> {
        if self.status == UpdateStatus::PendingValidation {
            self.boot_attempts += 1;
            if self.boot_attempts > self.max_attempts {
                self.status = UpdateStatus::RollbackRequired;
                self.rollback();
                return Err("Boot attempts exceeded, automatic rollback triggered");
            }
        }
        Ok(())
    }

    pub fn rollback(&mut self) {
        self.target_slot = None;
        self.status = UpdateStatus::Committed;
        self.boot_attempts = 0;
        crate::kprintln!("[UPDATE] Rollback completed to active slot {:?}", self.active_slot);
    }
}
