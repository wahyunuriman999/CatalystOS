// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

pub mod frame_allocator;
pub mod heap;
pub mod user;
pub mod address_space;

use bootloader_api::BootInfo;
use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
};

use crate::kprintln;

use core::sync::atomic::{AtomicU64, Ordering};

static PHYS_OFFSET: AtomicU64 = AtomicU64::new(0);

pub fn physical_offset() -> u64 {
    PHYS_OFFSET.load(Ordering::Relaxed)
}

pub fn init(boot_info: &'static BootInfo) -> OffsetPageTable<'static> {
    let phys_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("Physical memory offset not provided by bootloader!");
        
    PHYS_OFFSET.store(phys_offset, Ordering::Relaxed);

    kprintln!("[M2] Initializing memory subsystem...");
    
    let mut mapper = unsafe { init_mapper(phys_offset) };
    
    kprintln!("[M2] Initializing frame allocator...");
    frame_allocator::FRAME_ALLOCATOR.lock().init(&boot_info.memory_regions, phys_offset);
    
    let stats = frame_allocator::FRAME_ALLOCATOR.lock();
    kprintln!("[M2] Frame Allocator: {} total frames, {} used, {} free", 
        stats.total_frames, stats.used_frames, stats.total_frames - stats.used_frames);
    drop(stats);

    kprintln!("[M2] Initializing kernel heap...");
    heap::init_heap(&mut mapper, &mut *frame_allocator::FRAME_ALLOCATOR.lock())
        .expect("Heap initialization failed");

    kprintln!("[M2] Memory subsystem initialized successfully.");
    
    // Ensure PML4 entry 0 has USER_ACCESSIBLE for our tests!
    unsafe {
        let l4_table = active_level_4_table(phys_offset);
        if !l4_table[0].is_unused() {
            let flags = l4_table[0].flags();
            l4_table[0].set_flags(flags | x86_64::structures::paging::PageTableFlags::USER_ACCESSIBLE);
        }
    }
    
    mapper
}

pub unsafe fn init_mapper(physical_memory_offset: u64) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, VirtAddr::new(physical_memory_offset))
    }
}

unsafe fn active_level_4_table(physical_memory_offset: u64) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    
    let phys = level_4_table_frame.start_address();
    let virt = VirtAddr::new(physical_memory_offset + phys.as_u64());
    
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    
    unsafe { &mut *page_table_ptr }
}
