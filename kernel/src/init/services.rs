// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use crate::ipc::EndpointId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Crashed,
    Restarting,
}

pub struct Service {
    pub id: u32,
    pub name: String,
    pub pid: Option<u64>,
    pub endpoint: Option<EndpointId>,
    pub state: ServiceState,
    pub auto_restart: bool,
    pub restart_count: u32,
}

pub struct ServiceManager {
    services: Vec<Service>,
    next_id: u32,
}

impl ServiceManager {
    pub const fn new() -> Self {
        ServiceManager {
            services: Vec::new(),
            next_id: 1,
        }
    }

    pub fn register(&mut self, name: &str, auto_restart: bool) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.services.push(Service {
            id,
            name: String::from(name),
            pid: None,
            endpoint: None,
            state: ServiceState::Stopped,
            auto_restart,
            restart_count: 0,
        });
        id
    }

    pub fn set_running(&mut self, id: u32, pid: u64, endpoint: EndpointId) -> Result<(), &'static str> {
        let s = self.services.iter_mut().find(|s| s.id == id).ok_or("Service not found")?;
        s.pid = Some(pid);
        s.endpoint = Some(endpoint);
        s.state = ServiceState::Running;
        Ok(())
    }

    pub fn notify_crash(&mut self, pid: u64) -> Option<u32> {
        for s in self.services.iter_mut() {
            if s.pid == Some(pid) {
                s.state = ServiceState::Crashed;
                s.pid = None;
                if s.auto_restart {
                    s.restart_count += 1;
                    s.state = ServiceState::Restarting;
                    return Some(s.id);
                }
            }
        }
        None
    }

    pub fn get_service_endpoint(&self, name: &str) -> Option<EndpointId> {
        self.services.iter()
            .find(|s| s.name == name && s.state == ServiceState::Running)
            .and_then(|s| s.endpoint)
    }

    pub fn list(&self) -> Vec<(u32, String, ServiceState, u32)> {
        self.services.iter()
            .map(|s| (s.id, s.name.clone(), s.state, s.restart_count))
            .collect()
    }
}

pub static SERVICE_MANAGER: Mutex<ServiceManager> = Mutex::new(ServiceManager::new());

pub fn init_services() {
    crate::kprintln!("[SERVICES] Initializing Microkernel Service Manager...");
    let mut sm = SERVICE_MANAGER.lock();
    sm.register("vfsd", true);
    sm.register("netd", true);
    sm.register("displayd", true);
    sm.register("inputd", true);
    sm.register("logd", true);
    crate::kprintln!("[SERVICES] 5 Core System Services registered with auto-restart policy.");
}
