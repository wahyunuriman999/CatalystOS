// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use crate::objects::ObjectId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceId(pub u32);

#[derive(Debug, Clone)]
pub struct SpatialNode {
    pub surface_id: u32,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub z_depth: i32,
    pub width: u32,
    pub height: u32,
    pub pinned: bool,
    pub bound_object: Option<ObjectId>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceSnapshot {
    pub id: u32,
    pub name: String,
    pub nodes: Vec<SpatialNode>,
    pub timestamp_tick: u64,
}

pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub active_nodes: Vec<SpatialNode>,
    pub history: Vec<WorkspaceSnapshot>,
}

impl Workspace {
    pub fn new(id: WorkspaceId, name: &str) -> Self {
        Workspace {
            id,
            name: String::from(name),
            active_nodes: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn add_node(&mut self, surface_id: u32, title: &str, x: i32, y: i32, w: u32, h: u32, obj: Option<ObjectId>) {
        self.active_nodes.push(SpatialNode {
            surface_id,
            title: String::from(title),
            x,
            y,
            z_depth: 0,
            width: w,
            height: h,
            pinned: false,
            bound_object: obj,
        });
    }

    pub fn capture_snapshot(&mut self, tick: u64) -> u32 {
        let snap_id = (self.history.len() as u32) + 1;
        self.history.push(WorkspaceSnapshot {
            id: snap_id,
            name: self.name.clone(),
            nodes: self.active_nodes.clone(),
            timestamp_tick: tick,
        });
        snap_id
    }

    pub fn restore_snapshot(&mut self, snapshot_id: u32) -> Result<(), &'static str> {
        let snap = self.history.iter().find(|s| s.id == snapshot_id).ok_or("Workspace snapshot not found")?;
        self.active_nodes = snap.nodes.clone();
        Ok(())
    }
}

pub struct WorkspaceManager {
    workspaces: Vec<Workspace>,
    active_workspace_id: Option<WorkspaceId>,
}

impl WorkspaceManager {
    pub const fn new() -> Self {
        WorkspaceManager {
            workspaces: Vec::new(),
            active_workspace_id: None,
        }
    }

    pub fn create_workspace(&mut self, name: &str) -> WorkspaceId {
        let id = WorkspaceId((self.workspaces.len() as u32) + 1);
        let ws = Workspace::new(id, name);
        self.workspaces.push(ws);
        if self.active_workspace_id.is_none() {
            self.active_workspace_id = Some(id);
        }
        id
    }

    pub fn get_active_mut(&mut self) -> Option<&mut Workspace> {
        let active_id = self.active_workspace_id?;
        self.workspaces.iter_mut().find(|w| w.id == active_id)
    }

    pub fn total_workspaces(&self) -> usize {
        self.workspaces.len()
    }
}

pub static WORKSPACE_MANAGER: Mutex<WorkspaceManager> = Mutex::new(WorkspaceManager::new());
