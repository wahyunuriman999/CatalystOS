// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use x86_64::structures::paging::{PageTable, PageTableFlags, PageTableIndex, PhysFrame, Size4KiB, FrameAllocator};
use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;

#[derive(Debug)]
pub struct AddressSpace {
    pml4_frame: PhysFrame<Size4KiB>,
}

impl AddressSpace {
    pub fn new() -> Option<Self> {
        let mut allocator = crate::memory::frame_allocator::FRAME_ALLOCATOR.lock();
        let frame = allocator.allocate_frame()?;
        drop(allocator);

        let phys_offset = crate::memory::physical_offset();
        let virt_addr = VirtAddr::new(frame.start_address().as_u64() + phys_offset);
        let new_pml4 = unsafe { &mut *virt_addr.as_mut_ptr::<PageTable>() };
        
        // Zero out the entire table first
        new_pml4.zero();

        // Copy kernel mappings from current active CR3 (Entries 256..512)
        let (active_cr3, _) = Cr3::read();
        let active_virt = VirtAddr::new(active_cr3.start_address().as_u64() + phys_offset);
        let active_pml4 = unsafe { &*active_virt.as_ptr::<PageTable>() };

        for i in 256..512 {
            new_pml4[i] = active_pml4[i].clone();
        }

        Some(Self {
            pml4_frame: frame,
        })
    }
    
    pub fn pml4_frame(&self) -> PhysFrame<Size4KiB> {
        self.pml4_frame
    }
    
    pub fn activate(&self) {
        let (current_cr3, flags) = Cr3::read();
        if current_cr3 != self.pml4_frame {
            unsafe {
                Cr3::write(self.pml4_frame, flags);
            }
        }
    }

    /// Map a virtual page in this address space to a physical frame.
    /// Allocates intermediate page tables as necessary.
    pub fn map_page(&mut self, virt: VirtAddr, frame: PhysFrame<Size4KiB>, flags: PageTableFlags) -> Result<(), &'static str> {
        let phys_offset = crate::memory::physical_offset();
        let pml4_virt = VirtAddr::new(self.pml4_frame.start_address().as_u64() + phys_offset);
        let pml4 = unsafe { &mut *pml4_virt.as_mut_ptr::<PageTable>() };

        // Helper closure to get or create next page table level
        let mut get_or_create_table = |entry_idx: PageTableIndex, table: &mut PageTable| -> Result<&mut PageTable, &'static str> {
            if table[entry_idx].is_unused() {
                let mut allocator = crate::memory::frame_allocator::FRAME_ALLOCATOR.lock();
                let new_frame = allocator.allocate_frame().ok_or("Out of memory for page table")?;
                drop(allocator);

                let next_virt = VirtAddr::new(new_frame.start_address().as_u64() + phys_offset);
                let next_table = unsafe { &mut *next_virt.as_mut_ptr::<PageTable>() };
                next_table.zero();

                table[entry_idx].set_frame(
                    new_frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
                );
            }
            
            let target_frame = table[entry_idx].frame().map_err(|_| "Invalid frame in entry")?;
            let next_virt = VirtAddr::new(target_frame.start_address().as_u64() + phys_offset);
            Ok(unsafe { &mut *next_virt.as_mut_ptr::<PageTable>() })
        };

        let p3 = get_or_create_table(virt.p4_index(), pml4)?;
        let p2 = get_or_create_table(virt.p3_index(), p3)?;
        let p1 = get_or_create_table(virt.p2_index(), p2)?;

        let p1_idx = virt.p1_index();
        if !p1[p1_idx].is_unused() {
            return Err("Virtual address already mapped");
        }

        p1[p1_idx].set_frame(frame, flags);
        Ok(())
    }

    /// Allocates and maps anonymous user memory for a given virtual range.
    pub fn map_user_range(&mut self, start: VirtAddr, size: usize, flags: PageTableFlags) -> Result<(), &'static str> {
        let pages = (size + 4095) / 4096;
        for i in 0..pages {
            let page_vaddr = start + (i * 4096) as u64;
            let mut allocator = crate::memory::frame_allocator::FRAME_ALLOCATOR.lock();
            let frame = allocator.allocate_frame().ok_or("Out of physical memory")?;
            drop(allocator);

            self.map_page(page_vaddr, frame, flags | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::PRESENT)?;
        }
        Ok(())
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        let mut allocator = crate::memory::frame_allocator::FRAME_ALLOCATOR.lock();
        allocator.deallocate_frame(self.pml4_frame);
    }
}
