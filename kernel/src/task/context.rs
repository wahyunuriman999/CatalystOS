// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

/// context_switch(old_rsp_ptr: *mut u64, new_rsp: u64)
///
/// Saves callee-saved registers (rbx, rbp, r12-r15) onto the current stack,
/// stores the resulting RSP into *old_rsp_ptr, loads new_rsp into RSP,
/// then restores callee-saved registers from the new stack and returns.
///
/// This is the heart of Catalyst OS multi-tasking.
#[unsafe(naked)]
pub unsafe extern "C" fn context_switch(old_rsp_ptr: *mut u64, new_rsp: u64) {
    core::arch::naked_asm!(
        // Save callee-saved registers onto current stack (System V AMD64 ABI)
        "pushfq", // Save RFLAGS
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // Save current RSP into *old_rsp_ptr (first argument = rdi)
        "mov [rdi], rsp",
        // Load new RSP from new_rsp (second argument = rsi)
        "mov rsp, rsi",
        // Restore callee-saved registers from new task stack
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        "popfq", // Restore RFLAGS of the new task
        // Return — jumps to the new task's saved RIP
        "ret",
    );
}
