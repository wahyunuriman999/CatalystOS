// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Document,
    Spreadsheet,
    Media,
    Code,
    SpatialScene,
    Stream,
    Directory,
    GenericBinary,
}

#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    pub name: String,
    pub path: String,
    pub mime_type: String,
    pub size_bytes: usize,
    pub created_tick: u64,
    pub modified_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipType {
    DerivedFrom,
    ReferencedBy,
    ParentOf,
    ChildOf,
    LivingLink,
    TemporalPredecessor,
}

#[derive(Debug, Clone)]
pub struct Relationship {
    pub target_id: ObjectId,
    pub rel_type: RelationshipType,
}

#[derive(Debug, Clone)]
pub struct ObjectSnapshot {
    pub version: u32,
    pub tick: u64,
    pub data_hash: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CatalystObject {
    pub id: ObjectId,
    pub obj_type: ObjectType,
    pub metadata: ObjectMetadata,
    pub capabilities: u8,
    pub relationships: Vec<Relationship>,
    pub snapshots: Vec<ObjectSnapshot>,
    pub current_data: Vec<u8>,
}

impl CatalystObject {
    pub fn new(id: ObjectId, name: &str, path: &str, obj_type: ObjectType, initial_data: &[u8]) -> Self {
        let initial_hash = Self::calculate_hash(initial_data);
        let snapshot = ObjectSnapshot {
            version: 1,
            tick: 0,
            data_hash: initial_hash,
            data: initial_data.to_vec(),
        };

        CatalystObject {
            id,
            obj_type,
            metadata: ObjectMetadata {
                name: String::from(name),
                path: String::from(path),
                mime_type: String::from("application/octet-stream"),
                size_bytes: initial_data.len(),
                created_tick: 0,
                modified_tick: 0,
            },
            capabilities: 0xFF, // Full read/write/share default
            relationships: Vec::new(),
            snapshots: alloc::vec![snapshot],
            current_data: initial_data.to_vec(),
        }
    }

    pub fn add_relationship(&mut self, target_id: ObjectId, rel_type: RelationshipType) {
        self.relationships.push(Relationship { target_id, rel_type });
    }

    pub fn create_snapshot(&mut self, tick: u64) -> u32 {
        let next_ver = (self.snapshots.len() as u32) + 1;
        let hash = Self::calculate_hash(&self.current_data);
        self.snapshots.push(ObjectSnapshot {
            version: next_ver,
            tick,
            data_hash: hash,
            data: self.current_data.clone(),
        });
        next_ver
    }

    pub fn restore_snapshot(&mut self, version: u32) -> Result<(), &'static str> {
        let snap = self.snapshots.iter().find(|s| s.version == version).ok_or("Snapshot version not found")?;
        self.current_data = snap.data.clone();
        self.metadata.size_bytes = self.current_data.len();
        Ok(())
    }

    fn calculate_hash(data: &[u8]) -> u32 {
        let mut sum: u32 = 0;
        for &b in data {
            sum = sum.wrapping_add(b as u32);
        }
        sum
    }
}

pub struct ObjectRegistry {
    objects: Vec<CatalystObject>,
    next_id: u64,
}

impl ObjectRegistry {
    pub const fn new() -> Self {
        ObjectRegistry {
            objects: Vec::new(),
            next_id: 1,
        }
    }

    pub fn register(&mut self, name: &str, path: &str, obj_type: ObjectType, data: &[u8]) -> ObjectId {
        let id = ObjectId(self.next_id);
        self.next_id += 1;
        let obj = CatalystObject::new(id, name, path, obj_type, data);
        self.objects.push(obj);
        id
    }

    pub fn get(&self, id: ObjectId) -> Option<&CatalystObject> {
        self.objects.iter().find(|o| o.id == id)
    }

    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut CatalystObject> {
        self.objects.iter_mut().find(|o| o.id == id)
    }

    pub fn total_objects(&self) -> usize {
        self.objects.len()
    }
}

pub static OBJECT_REGISTRY: Mutex<ObjectRegistry> = Mutex::new(ObjectRegistry::new());
