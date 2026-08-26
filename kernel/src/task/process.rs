// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::sync::Arc;
use spin::Mutex;
use crate::memory::address_space::AddressSpace;
use crate::ipc::EndpointId;

static NEXT_PID: AtomicU64 = AtomicU64::new(1);
static NEXT_TID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockReason {
    ReceiveIPC(EndpointId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked(BlockReason),
    Sleeping,
    Dead,
}

pub const STACK_SIZE: usize = 16384;

#[repr(C, align(16))]
pub struct TaskStack(pub [u8; STACK_SIZE]);

pub struct Process {
    pub pid: u64,
    pub address_space: Option<Arc<AddressSpace>>,
}

impl Process {
    pub fn new(pid: u64) -> Self {
        Process {
            pid,
            address_space: Some(Arc::new(AddressSpace::new().unwrap())),
        }
    }
}

pub struct Task {
    pub tid: u64,
    pub process: Arc<Process>,
    pub name: &'static str,
    pub state: TaskState,
    pub stack: alloc::boxed::Box<TaskStack>,
    pub stack_pointer: u64,
    pub priority: u8,
}

impl Task {
    pub fn new_idle() -> Self {
        let stack = alloc::boxed::Box::new(TaskStack([0u8; STACK_SIZE]));
        let process = Arc::new(Process {
            pid: 0,
            address_space: None, // Kernel task
        });
        Task {
            tid: 0,
            process,
            name: "idle",
            state: TaskState::Running,
            stack,
            stack_pointer: 0,
            priority: 255,
        }
    }

    pub fn new(name: &'static str, entry_point: fn() -> !, priority: u8) -> Self {
        let tid = NEXT_TID.fetch_add(1, Ordering::Relaxed);
        let mut stack = alloc::boxed::Box::new(TaskStack([0u8; STACK_SIZE]));
        
        let process = Arc::new(Process {
            pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
            address_space: None,
        });

        let stack_base = stack.0.as_mut_ptr() as usize;
        let stack_top = stack_base + STACK_SIZE;
        let frame_start = (stack_top - 8 * 8) as *mut u64;

        unsafe {
            *frame_start.add(0) = entry_point as u64;
            *frame_start.add(1) = 0u64;
            *frame_start.add(2) = 0u64;
            *frame_start.add(3) = 0u64;
            *frame_start.add(4) = 0u64;
            *frame_start.add(5) = 0u64;
            *frame_start.add(6) = 0x202;
            *frame_start.add(7) = task_entry as *const () as u64;
        }

        let stack_pointer = frame_start as u64;

        Task { tid, process, name, state: TaskState::Ready, stack, stack_pointer, priority }
    }
}

/// Called when a task first starts. r15 = entry_point fn pointer.
#[unsafe(naked)]
pub unsafe extern "C" fn task_entry() -> ! {
    core::arch::naked_asm!(
        "mov rdi, r15",
        "call task_call_entry",
        "ud2",
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn task_call_entry(entry: u64) -> ! {
    let f: fn() -> ! = core::mem::transmute(entry as usize);
    f();
}

#[unsafe(naked)]
pub unsafe extern "C" fn enter_usermode(entry_point: u64, stack_pointer: u64) -> ! {
    core::arch::naked_asm!(
        // We'll use iretq to transition to Ring 3.
        // We need to push: SS, RSP, RFLAGS, CS, RIP
        
        // 1. push SS (user data selector = 0x23)
        "mov rax, 0x23",
        "push rax",
        
        // 2. push RSP
        "push rsi",
        
        // 3. push RFLAGS (with interrupts enabled, bit 9)
        "pushfq",
        "pop rax",
        "or rax, 512",
        "push rax",
        
        // 4. push CS (user code selector = 0x2B)
        "mov rax, 0x2b",
        "push rax",
        
        // 5. push RIP
        "push rdi",
        
        // 6. clear registers to avoid leaking kernel state
        "xor rax, rax",
        "xor rbx, rbx",
        "xor rcx, rcx",
        "xor rdx, rdx",
        "xor rsi, rsi",
        "xor rdi, rdi",
        "xor rbp, rbp",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",
        "xor r11, r11",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",
        
        // 7. jump to usermode
        "iretq",
    );
}




