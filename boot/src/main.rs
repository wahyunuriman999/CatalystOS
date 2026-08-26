// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

//! Catalyst OS Boot Image Builder
//!
//! This binary is NOT the bootloader itself. It uses the `bootloader` crate
//! to create a bootable disk image from the kernel binary.
//! Run with: `cargo run` (from the boot/ directory)

use std::path::PathBuf;

fn main() {
    // Locate the kernel binary built for the custom target
    let kernel_path = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // go up from boot/ to workspace root
        path.push("target");
        path.push("x86_64-catalyst");
        path.push("debug");
        path.push("catalyst-kernel");
        path
    };

    // Build the bootable disk image
    let uefi_path = kernel_path.with_extension("img");
    
    bootloader::UefiBoot::new(&kernel_path)
        .create_disk_image(&uefi_path)
        .expect("Failed to create UEFI disk image");
    
    println!("Created UEFI boot image: {}", uefi_path.display());
}
