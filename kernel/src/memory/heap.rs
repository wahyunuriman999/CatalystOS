use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use spin::Mutex;
use x86_64::structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 32 * 1024 * 1024; // 32 MiB

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }
    
    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

pub struct Locked<A> {
    inner: Mutex<A>,
}
impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked { inner: Mutex::new(inner) }
    }
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();
        
        let alloc_start = (bump.next + layout.align() - 1) & !(layout.align() - 1);
        let alloc_end = alloc_start.checked_add(layout.size()).unwrap_or(0);
        
        if alloc_end > bump.heap_end || alloc_end == 0 {
            null_mut() // out of memory
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();
        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

#[global_allocator]
pub static HEAP: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + (HEAP_SIZE as u64) - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    HEAP.lock().init(HEAP_START, HEAP_SIZE);

    Ok(())
}
