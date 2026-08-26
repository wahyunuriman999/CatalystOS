use x86_64::structures::paging::{PageTable, PhysFrame, Size4KiB, FrameAllocator};
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
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        // We should free all user pages here!
        // For now, we at least free the PML4 frame.
        let mut allocator = crate::memory::frame_allocator::FRAME_ALLOCATOR.lock();
        allocator.deallocate_frame(self.pml4_frame);
    }
}
