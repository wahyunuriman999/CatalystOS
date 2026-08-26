// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

const BITMAP_SIZE: usize = 16384; // Supports up to 4GB of physical memory (16384 * 64 * 4KB)

pub struct BitmapFrameAllocator {
    bitmap: [u64; BITMAP_SIZE],
    next_free: usize,
    pub total_frames: usize,
    pub used_frames: usize,
    phys_offset: u64,
}

impl BitmapFrameAllocator {
    pub const fn new() -> Self {
        BitmapFrameAllocator {
            bitmap: [!0; BITMAP_SIZE],
            next_free: 0,
            total_frames: 0,
            used_frames: 0,
            phys_offset: 0,
        }
    }

    pub fn init(&mut self, memory_regions: &MemoryRegions, phys_offset: u64) {
        self.total_frames = 0;
        self.used_frames = 0;
        self.phys_offset = phys_offset;

        for region in memory_regions.iter() {
            let start_frame = region.start / 4096;
            let end_frame = region.end / 4096;

            if region.kind == MemoryRegionKind::Usable {
                for frame in start_frame..end_frame {
                    self.total_frames += 1;
                    self.mark_free(frame as usize);
                }
            } else {
                for frame in start_frame..end_frame {
                    self.total_frames += 1;
                    self.mark_used(frame as usize);
                    self.used_frames += 1;
                }
            }
        }
    }

    fn mark_free(&mut self, frame: usize) {
        if frame < BITMAP_SIZE * 64 {
            let idx = frame / 64;
            let bit = frame % 64;
            self.bitmap[idx] &= !(1 << bit);
        }
    }

    fn mark_used(&mut self, frame: usize) {
        if frame < BITMAP_SIZE * 64 {
            let idx = frame / 64;
            let bit = frame % 64;
            self.bitmap[idx] |= 1 << bit;
        }
    }

    pub fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let frame_idx = (frame.start_address().as_u64() / 4096) as usize;
        if frame_idx < BITMAP_SIZE * 64 {
            let array_idx = frame_idx / 64;
            let bit_idx = frame_idx % 64;
            let bit_mask = 1 << bit_idx;
            
            if (self.bitmap[array_idx] & bit_mask) != 0 {
                self.bitmap[array_idx] &= !bit_mask;
                self.used_frames = self.used_frames.saturating_sub(1);
                if frame_idx < self.next_free {
                    self.next_free = frame_idx;
                }
            } else {
                panic!("[FATAL] Double free or attempt to free unallocated/reserved frame: {:#x}", frame.start_address().as_u64());
            }
        } else {
            panic!("[FATAL] Attempt to free out of bounds frame: {:#x}", frame.start_address().as_u64());
        }
    }
    pub fn free_frames(&self) -> usize {
        let mut count = 0;
        for i in 0..BITMAP_SIZE {
            let chunk = self.bitmap[i];
            if chunk != u64::MAX {
                count += chunk.count_zeros() as usize;
            }
        }
        count
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        for idx in self.next_free..(BITMAP_SIZE * 64) {
            let array_idx = idx / 64;
            let bit_idx = idx % 64;

            if (self.bitmap[array_idx] & (1 << bit_idx)) == 0 {
                self.mark_used(idx);
                self.used_frames += 1;
                self.next_free = idx + 1;

                let phys_addr = (idx as u64) * 4096;
                // ZERO THE ALLOCATED FRAME to prevent garbage page tables!
                if self.phys_offset != 0 {
                    let virt_addr = phys_addr + self.phys_offset;
                    unsafe {
                        core::ptr::write_bytes(virt_addr as *mut u8, 0, 4096);
                    }
                }

                return Some(PhysFrame::containing_address(PhysAddr::new(phys_addr)));
            }
        }
        None
    }
}

pub static FRAME_ALLOCATOR: Mutex<BitmapFrameAllocator> = Mutex::new(BitmapFrameAllocator::new());
