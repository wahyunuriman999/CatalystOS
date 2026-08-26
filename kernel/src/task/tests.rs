use core::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER_A: AtomicU64 = AtomicU64::new(0);
static TEST_COUNTER_B: AtomicU64 = AtomicU64::new(0);

fn task_a() -> ! {
    crate::kprintln!("[TEST] Task A started.");
    loop {
        TEST_COUNTER_A.fetch_add(1, Ordering::Relaxed);
        // Busy wait to simulate work and force PREEMPTION (Test B/D)
        for _ in 0..100000 {
            unsafe { core::arch::asm!("nop") };
        }
        
        // Test C: IF Preservation
        // Enable interrupts, yield, and verify they are still enabled upon return
        unsafe {
            x86_64::instructions::interrupts::enable();
            crate::task::scheduler::do_schedule(); // voluntary yield
            
            let flags: u64;
            core::arch::asm!(
                "pushfq",
                "pop {}",
                out(reg) flags
            );
            if (flags & (1 << 9)) == 0 {
                crate::kprintln!("[FATAL] Task A lost IF after voluntary yield!");
            }
        }
    }
}

fn task_b() -> ! {
    crate::kprintln!("[TEST] Task B started.");
    loop {
        TEST_COUNTER_B.fetch_add(1, Ordering::Relaxed);
        // Busy wait
        for _ in 0..100000 {
            unsafe { core::arch::asm!("nop") };
        }
        
        // Print progress every ~100 loops
        let b = TEST_COUNTER_B.load(Ordering::Relaxed);
        if b % 100 == 0 {
            let a = TEST_COUNTER_A.load(Ordering::Relaxed);
            crate::kprintln!("[TEST VERIFICATION] Task A: {}, Task B: {} (Preemption Active)", a, b);
        }
    }
}

pub fn spawn_verification_tasks() {
    crate::task::scheduler::spawn("test_a", task_a, 1);
    crate::task::scheduler::spawn("test_b", task_b, 1);
}
