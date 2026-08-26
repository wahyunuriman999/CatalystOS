// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use spin::Mutex;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::task::process::{Task, TaskState};

pub static SCHEDULE_NEEDED: AtomicBool = AtomicBool::new(false);

pub const MAX_TASKS: usize = 256;

pub struct Scheduler {
    pub tasks: VecDeque<Task>,
    /// Stack pointer save location for the currently-running task
    pub current_sp_ptr: u64,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            tasks: VecDeque::new(),
            current_sp_ptr: 0,
        }
    }

    pub fn add_task(&mut self, task: Task) -> Result<(), &'static str> {
        if self.tasks.len() >= MAX_TASKS {
            return Err("Run queue capacity exceeded");
        }
        self.tasks.push_back(task);
        Ok(())
    }

    pub fn is_task_alive(&self, tid: u64) -> bool {
        for t in self.tasks.iter() {
            if t.tid == tid && t.state != TaskState::Dead {
                return true;
            }
        }
        false
    }

    /// Round-robin: rotate to the next Ready task.
    /// Returns (old_sp_ptr, new_sp, cr3_opt, kernel_stack_top) or None if only one task.
    pub fn next_task(&mut self) -> Option<(u64, u64, Option<x86_64::structures::paging::PhysFrame>, u64)> {
        if self.tasks.len() < 2 {
            return None;
        }

        if let Some(front) = self.tasks.front_mut() {
            if front.state == TaskState::Running {
                front.state = TaskState::Ready;
            }
        }

        let len = self.tasks.len();
        let mut found = false;
        for _ in 0..len {
            if let Some(task) = self.tasks.front() {
                if task.state == TaskState::Ready {
                    found = true;
                    break;
                }
            }
            if let Some(t) = self.tasks.pop_front() {
                self.tasks.push_back(t);
            }
        }

        if !found {
            return None;
        }

        if let Some(mut old_task) = self.tasks.pop_front() {
            let old_sp_ptr = &mut old_task.stack_pointer as *mut u64 as u64;
            let (new_sp, cr3, stack_top) = if let Some(new_task) = self.tasks.front_mut() {
                new_task.state = TaskState::Running;
                let cr3 = new_task.process.address_space.as_ref().map(|aspace| aspace.pml4_frame());
                let stack_top = new_task.stack.0.as_ptr() as u64 + crate::task::process::STACK_SIZE as u64;
                (new_task.stack_pointer, cr3, stack_top)
            } else {
                self.tasks.push_back(old_task);
                return None;
            };
            self.tasks.push_back(old_task);
            Some((old_sp_ptr, new_sp, cr3, stack_top))
        } else {
            None
        }
    }
    
    pub fn reap_dead_tasks(&mut self) {
        self.tasks.retain(|t| t.state != TaskState::Dead);
    }
}

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

pub fn init() {
    let idle = Task::new_idle();
    SCHEDULER.lock().add_task(idle).expect("Failed to add idle task");
    crate::kprintln!("[SCHED] Preemptive scheduler initialized.");
}

pub fn spawn(name: &'static str, entry: fn() -> !, priority: u8) {
    let task = Task::new(name, entry, priority);
    crate::kprintln!("[SCHED] Spawned task '{}' (TID {})", task.name, task.tid);
    SCHEDULER.lock().add_task(task).expect("Failed to spawn task");
}

pub fn terminate_current_thread() -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut sched = SCHEDULER.lock();
        if let Some(front) = sched.tasks.front_mut() {
            crate::kprintln!("[SCHED] Terminating TID {}", front.tid);
            front.state = TaskState::Dead;
        }
    });
    
    // We defer stack/process cleanup here for a real deferred reaper,
    // ensuring we don't free the kernel stack while executing on it!
    
    loop {
        do_schedule();
    }
}

pub fn do_schedule() {
    let result = x86_64::instructions::interrupts::without_interrupts(|| {
        let mut sched = SCHEDULER.lock();
        sched.reap_dead_tasks(); // Deferred reaping of Dead tasks!
        sched.next_task()
    });

    if let Some((old_sp_ptr, new_sp, cr3_opt, kernel_stack_top)) = result {
        unsafe {
            crate::arch::syscall::CPU_LOCAL.kernel_rsp = kernel_stack_top;
            crate::arch::gdt::set_rsp0(kernel_stack_top);
            
            if let Some(cr3) = cr3_opt {
                let (current_cr3, flags) = x86_64::registers::control::Cr3::read();
                if current_cr3 != cr3 {
                    x86_64::registers::control::Cr3::write(cr3, flags);
                }
            }
            
            crate::task::context::context_switch(old_sp_ptr as *mut u64, new_sp);
        }
    }
}
