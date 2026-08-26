use x86_64::structures::paging::{Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

pub fn alloc_user_pages(start_addr: u64, size: usize, physical_memory_offset: u64) {
    let mut mapper = unsafe { crate::memory::init_mapper(physical_memory_offset) };
    
    let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(start_addr));
    let end_page = Page::containing_address(VirtAddr::new(start_addr + size as u64 - 1));
    
    for page in Page::range_inclusive(start_page, end_page) {
        // We only want to UPDATE the flags to include USER_ACCESSIBLE
        unsafe {
            if let Ok(mut frame) = mapper.update_flags(page, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE) {
                frame.flush();
            }
        }
    }
}
