// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionKind {
    FileSystemPath(String),
    NetworkSocket,
    DisplaySurface,
    AudioOutput,
    HardwareDevice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Deny,
    PromptUser,
}

#[derive(Debug, Clone)]
pub struct PermissionRule {
    pub kind: PermissionKind,
    pub decision: PermissionDecision,
}

pub struct PermissionBubble {
    pub pid: u64,
    pub rules: Vec<PermissionRule>,
    pub isolated: bool,
}

impl PermissionBubble {
    pub fn new(pid: u64) -> Self {
        PermissionBubble {
            pid,
            rules: Vec::new(),
            isolated: true,
        }
    }

    pub fn grant(&mut self, kind: PermissionKind) {
        self.rules.retain(|r| r.kind != kind);
        self.rules.push(PermissionRule {
            kind,
            decision: PermissionDecision::Allow,
        });
    }

    pub fn deny(&mut self, kind: PermissionKind) {
        self.rules.retain(|r| r.kind != kind);
        self.rules.push(PermissionRule {
            kind,
            decision: PermissionDecision::Deny,
        });
    }

    pub fn check(&self, kind: &PermissionKind) -> PermissionDecision {
        for rule in self.rules.iter() {
            if &rule.kind == kind {
                return rule.decision;
            }
        }
        // Default to PromptUser (Zero Ambient Authority)
        PermissionDecision::PromptUser
    }
}

pub struct BubbleManager {
    bubbles: Vec<PermissionBubble>,
}

impl BubbleManager {
    pub const fn new() -> Self {
        BubbleManager {
            bubbles: Vec::new(),
        }
    }

    pub fn get_or_create(&mut self, pid: u64) -> &mut PermissionBubble {
        if let Some(pos) = self.bubbles.iter().position(|b| b.pid == pid) {
            return &mut self.bubbles[pos];
        }
        self.bubbles.push(PermissionBubble::new(pid));
        let last_idx = self.bubbles.len() - 1;
        &mut self.bubbles[last_idx]
    }
}

pub static BUBBLE_MANAGER: Mutex<BubbleManager> = Mutex::new(BubbleManager::new());
