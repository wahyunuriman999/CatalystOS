#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![feature(naked_functions)]

extern crate alloc;

pub mod console;
pub mod memory;
pub mod arch;
pub mod task;
pub mod compat;
pub mod drivers;
pub mod graphics;
pub mod events;
pub mod input;
pub mod test_harness;
pub mod ipc;
pub mod storage;

use bootloader_api::{entry_point, BootInfo};

pub static BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config.mappings.page_table_recursive = Some(bootloader_api::config::Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    console::init();
    crate::kprintln!("SERIAL INITIALIZED!");
    
    // Extract framebuffer info FIRST before passing boot_info to memory::init
    let mut fb_ptr = core::ptr::null_mut();
    let mut fb_len = 0;
    let mut fb_info = None;
    
    if let Some(fb) = boot_info.framebuffer.as_mut() {
        let info = fb.info();
        fb_info = Some(info);
        fb_ptr = fb.buffer_mut().as_mut_ptr();
        fb_len = fb.buffer_mut().len();
        unsafe {
            let fb_slice = core::slice::from_raw_parts_mut(fb_ptr, fb_len);
            for i in 0..fb_len { fb_slice[i] = 100; }
            console::init_vga(fb_slice, info);
        }
    }
    crate::kprintln!("VGA INITIALIZED!");

    crate::kprintln!("Initializing memory...");
    memory::init(boot_info);
    
    crate::kprintln!("Initializing architecture (GDT, IDT, PICs)...");
    arch::init();

    storage::init();
    
    crate::kprintln!("---------- M6: Catalyst GUI Desktop ----------");
    if let Some(info) = fb_info {
        unsafe {
            let fb_slice = core::slice::from_raw_parts_mut(fb_ptr, fb_len);
            crate::graphics::canvas::store_framebuffer(fb_slice, &info);
        }
        
        crate::graphics::init_gpu();
        
        // Disable text console so it doesn't overwrite GUI
        if let Some(writer) = &mut *crate::console::vga::FRAMEBUFFER_WRITER.lock() {
            writer.disabled = true;
        }
        
        crate::task::init();
        
        // Phase 3 Runtime Verification
        crate::test_harness::run_all_tests();
        
        // Start interrupt-driven scheduler
        x86_64::instructions::interrupts::enable();
        loop {
            x86_64::instructions::hlt();
        }
    } else {
        panic!("NO FRAMEBUFFER PROVIDED BY BOOTLOADER!");
    }
}

fn desktop_task() -> ! {
    let mut input_dispatcher = crate::input::InputDispatcher::new();
    let mut keyboard_count = 0;
    let mut mouse_count = 0;
    let mut last_tick = 0;
    
    loop {
        // Check timer for metrics
        let current_tick = crate::arch::interrupts::tick_count();
        if current_tick - last_tick >= 1000 { // 10 seconds at 100Hz
            let q_push = crate::events::EVENT_QUEUE.push_count.load(core::sync::atomic::Ordering::Relaxed);
            let q_drop = crate::events::EVENT_QUEUE.drop_count.load(core::sync::atomic::Ordering::Relaxed);
            let q_pop = crate::events::EVENT_QUEUE.pop_count.load(core::sync::atomic::Ordering::Relaxed);
            
            let (windows, merged, redraws, cursors, moves, hits, captures) = crate::graphics::desktop::DESKTOP.lock().get_metrics();
            
            crate::kprintln!(
                "[WINDOWING VERIFICATION] Tick: {}\n  Queue: Pushed: {}, Popped: {}, Dropped: {}\n  Input: Keys: {}, Mouse: {}\n  Windows: {}, Merges: {}, Full Redraws: {}\n  Cursor Invals: {}, Moves: {}, Hit Tests: {}, Captures: {}", 
                current_tick, q_push, q_pop, q_drop, keyboard_count, mouse_count,
                windows, merged, redraws, cursors, moves, hits, captures
            );
            last_tick = current_tick;
        }

        let mut handled_any = false;
        // Process events
        while let Some(event) = crate::events::pop_event() {
            handled_any = true;
            input_dispatcher.process_event(event, |generic_event| {
                match generic_event {
                    crate::events::InputEvent::KeyDown { .. } | crate::events::InputEvent::KeyUp { .. } => {
                        keyboard_count += 1;
                        crate::graphics::desktop::DESKTOP.lock().handle_event(generic_event);
                    },
                    crate::events::InputEvent::MouseMove { .. } |
                    crate::events::InputEvent::MouseButtonDown { .. } |
                    crate::events::InputEvent::MouseButtonUp { .. } => {
                        mouse_count += 1;
                        crate::graphics::desktop::DESKTOP.lock().handle_event(generic_event);
                    },
                    _ => {}
                }
            });
        }
        
        // Render only if dirty
        let is_dirty = crate::graphics::desktop::DESKTOP.lock().is_dirty();
        if is_dirty {
            let mut desktop = crate::graphics::desktop::DESKTOP.lock();
            crate::graphics::canvas::with_canvas(|c| {
                desktop.draw_if_dirty(c);
            });
        }
        
        // Sleep if no work
        if !handled_any && !is_dirty {
            // We use yield_now instead of hlt, because hlt is for the idle thread
            crate::task::scheduler::do_schedule(); // Cooperative yield
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { 
        crate::console::SERIAL.force_unlock(); 
        crate::console::vga::FRAMEBUFFER_WRITER.force_unlock(); 
    }
    crate::kprintln!("PANIC: {}", info);
    loop { x86_64::instructions::hlt(); }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

