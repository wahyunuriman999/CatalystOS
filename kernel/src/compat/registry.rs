// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::collections::BTreeMap;
use alloc::string::String;
use spin::Mutex;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum RegValue {
    Dword(u32),
    Qword(u64),
    Sz(String),
    Binary(alloc::vec::Vec<u8>),
}

pub static REGISTRY: Mutex<BTreeMap<String, RegValue>> = Mutex::new(BTreeMap::new());

#[allow(dead_code)]
pub fn reg_set(key: &str, value: RegValue) {
    REGISTRY.lock().insert(String::from(key), value);
}

#[allow(dead_code)]
pub fn reg_get(key: &str) -> Option<RegValue> {
    REGISTRY.lock().get(key).cloned()
}

pub fn init_default_keys() {
    // Populate with Windows-compatible default keys
    reg_set("HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProductName",
        RegValue::Sz(String::from("Catalyst OS")));
    reg_set("HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\CurrentVersion",
        RegValue::Sz(String::from("10.0")));
    reg_set("HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\CurrentBuildNumber",
        RegValue::Sz(String::from("19041")));
    reg_set("HKLM\\SYSTEM\\CurrentControlSet\\Control\\ComputerName\\ComputerName",
        RegValue::Sz(String::from("CATALYST")));
    reg_set("HKLM\\SOFTWARE\\CatalystOS\\Version",
        RegValue::Sz(String::from("0.0.5")));
    crate::kprintln!("[REGISTRY] Windows compatibility registry initialized.");
}
