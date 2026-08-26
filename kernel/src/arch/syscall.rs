// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use x86_64::registers::model_specific::{Efer, EferFlags, LStar, Star, SFMask, KernelGsBase};
use x86_64::instructions::segmentation::Segment;
use x86_64::structures::gdt::SegmentSelector;
use core::arch::naked_asm;
use crate::storage::vfs::{vfs_open, vfs_mkdir, vfs_unlink, OpenFile, VNodeType};
use crate::ipc::{IPC_REGISTRY, CapabilityTable, CapabilityHandle, CAP_SEND, CAP_RECEIVE, CAP_CALL, cap_send, cap_receive, cap_call, cap_reply, EndpointId};
use crate::memory::user::{validate_user_buffer, copy_from_user, copy_to_user};

// Syscall Numbers
pub const SYS_EXIT: u64          = 1;
pub const SYS_YIELD: u64         = 2;
pub const SYS_GETPID: u64        = 4;
pub const SYS_KILL: u64          = 5;
pub const SYS_OPEN: u64          = 10;
pub const SYS_CLOSE: u64         = 11;
pub const SYS_READ: u64          = 12;
pub const SYS_WRITE: u64         = 13;
pub const SYS_MKDIR: u64         = 16;
pub const SYS_UNLINK: u64        = 17;
pub const SYS_GETCWD: u64        = 18;
pub const SYS_CHDIR: u64         = 19;
pub const SYS_IPC_CREATE_EP: u64 = 20;
pub const SYS_IPC_DESTROY_EP: u64= 21;
pub const SYS_IPC_SEND: u64      = 24;
pub const SYS_IPC_RECEIVE: u64   = 25;
pub const SYS_IPC_CALL: u64      = 26;
pub const SYS_IPC_REPLY: u64     = 27;
pub const SYS_SPAWN: u64         = 30;
pub const SYS_WAIT: u64          = 31;

#[repr(C)]
pub struct CpuLocal {
    pub kernel_rsp: u64,
    pub user_rsp: u64,
}

#[unsafe(no_mangle)]
pub static mut CPU_LOCAL: CpuLocal = CpuLocal { kernel_rsp: 0, user_rsp: 0 };

pub fn init(
    kernel_code: SegmentSelector,
    _kernel_data: SegmentSelector,
    user_code_32: SegmentSelector,
    _user_data: SegmentSelector
) {
    unsafe {
        let priv_stack_top = 0x200000;
        CPU_LOCAL.kernel_rsp = priv_stack_top; 
        KernelGsBase::write(x86_64::VirtAddr::new(&raw const CPU_LOCAL as u64));

        Efer::update(|flags| flags.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
        
        let mut star_msr = x86_64::registers::model_specific::Msr::new(0xC000_0081);
        let mut star_val = 0u64;
        star_val |= (kernel_code.0 as u64) << 32;
        star_val |= ((user_code_32.0 as u64) | 3) << 48;
        star_msr.write(star_val);
        
        LStar::write(x86_64::VirtAddr::new(syscall_entry as *const () as u64));
        
        SFMask::write(x86_64::registers::rflags::RFlags::INTERRUPT_FLAG);
    }
    crate::kprintln!("[SYSCALL] System calls initialized.");
}

#[unsafe(naked)]
extern "C" fn syscall_entry() {
    unsafe {
        naked_asm!(
            "swapgs",
            "mov gs:[8], rsp",
            "mov rsp, gs:[0]",

            "push rcx",
            "push r11",
            "push rbp",
            "mov rbp, rsp",
              
            "mov rdi, rax",
            "mov rsi, r10",
            "mov rcx, r8",
            "mov r8, r9",
              
            "call {}",
              
            "mov rsp, rbp",
            "pop rbp",
            "pop r11",
            "pop rcx",
            
            "mov rsp, gs:[8]",
            "swapgs",
            "sysretq",
            sym syscall_handler
        );
    }
}

extern "C" fn syscall_handler(sys_no: u64, arg1: u64, arg2: u64, arg3: u64, _arg4: u64) -> u64 {
    match sys_no {
        SYS_EXIT => {
            crate::kprintln!("[SYSCALL] sys_exit called with code {}", arg1);
            crate::task::scheduler::terminate_current_thread();
        }
        SYS_YIELD => {
            crate::task::scheduler::do_schedule();
            0
        }
        SYS_GETPID => {
            let sched = crate::task::scheduler::SCHEDULER.lock();
            sched.current_tid().unwrap_or(0)
        }
        SYS_WRITE => {
            // arg1 = fd, arg2 = user_buf_ptr, arg3 = len
            let fd = arg1 as usize;
            let ptr = arg2 as *const u8;
            let len = arg3 as usize;

            if let Err(_) = validate_user_buffer(arg2, len) {
                return u64::MAX; // -EFAULT
            }

            if fd == 1 || fd == 2 {
                // stdout / stderr
                let mut buf = alloc::vec![0u8; len];
                if copy_from_user(ptr, &mut buf).is_ok() {
                    if let Ok(s) = core::str::from_utf8(&buf) {
                        crate::kprint!("{}", s);
                    }
                    return len as u64;
                }
                return u64::MAX;
            }

            // Regular file write
            let mut sched = crate::task::scheduler::SCHEDULER.lock();
            if let Some(task) = sched.tasks.front() {
                let mut files = task.process.files.lock();
                if let Ok(open_file) = files.get(fd) {
                    let mut buf = alloc::vec![0u8; len];
                    if copy_from_user(ptr, &mut buf).is_ok() {
                        if let Ok(written) = open_file.vnode.write(open_file.offset, &buf) {
                            open_file.offset += written;
                            return written as u64;
                        }
                    }
                }
            }
            u64::MAX
        }
        SYS_READ => {
            let fd = arg1 as usize;
            let ptr = arg2 as *mut u8;
            let len = arg3 as usize;

            if let Err(_) = validate_user_buffer(arg2, len) {
                return u64::MAX;
            }

            let mut sched = crate::task::scheduler::SCHEDULER.lock();
            if let Some(task) = sched.tasks.front() {
                let mut files = task.process.files.lock();
                if let Ok(open_file) = files.get(fd) {
                    let mut buf = alloc::vec![0u8; len];
                    if let Ok(bytes_read) = open_file.vnode.read(open_file.offset, &mut buf) {
                        if copy_to_user(&buf[..bytes_read], ptr).is_ok() {
                            open_file.offset += bytes_read;
                            return bytes_read as u64;
                        }
                    }
                }
            }
            u64::MAX
        }
        SYS_OPEN => {
            // arg1 = path_ptr, arg2 = path_len, arg3 = flags
            let path_ptr = arg1 as *const u8;
            let path_len = arg2 as usize;
            let flags = arg3 as u32;

            if let Err(_) = validate_user_buffer(arg1, path_len) {
                return u64::MAX;
            }

            let mut path_buf = alloc::vec![0u8; path_len];
            if copy_from_user(path_ptr, &mut path_buf).is_err() {
                return u64::MAX;
            }

            if let Ok(path_str) = core::str::from_utf8(&path_buf) {
                if let Ok(vnode) = vfs_open(path_str, flags) {
                    let open_file = OpenFile { vnode, offset: 0, flags };
                    let mut sched = crate::task::scheduler::SCHEDULER.lock();
                    if let Some(task) = sched.tasks.front() {
                        let mut files = task.process.files.lock();
                        if let Ok(fd) = files.insert(open_file) {
                            return fd as u64;
                        }
                    }
                }
            }
            u64::MAX
        }
        SYS_CLOSE => {
            let fd = arg1 as usize;
            let mut sched = crate::task::scheduler::SCHEDULER.lock();
            if let Some(task) = sched.tasks.front() {
                let mut files = task.process.files.lock();
                if files.close(fd).is_ok() {
                    return 0;
                }
            }
            u64::MAX
        }
        SYS_MKDIR => {
            let path_ptr = arg1 as *const u8;
            let path_len = arg2 as usize;
            if let Err(_) = validate_user_buffer(arg1, path_len) {
                return u64::MAX;
            }
            let mut path_buf = alloc::vec![0u8; path_len];
            if copy_from_user(path_ptr, &mut path_buf).is_err() {
                return u64::MAX;
            }
            if let Ok(path_str) = core::str::from_utf8(&path_buf) {
                if vfs_mkdir(path_str).is_ok() {
                    return 0;
                }
            }
            u64::MAX
        }
        SYS_UNLINK => {
            let path_ptr = arg1 as *const u8;
            let path_len = arg2 as usize;
            if let Err(_) = validate_user_buffer(arg1, path_len) {
                return u64::MAX;
            }
            let mut path_buf = alloc::vec![0u8; path_len];
            if copy_from_user(path_ptr, &mut path_buf).is_err() {
                return u64::MAX;
            }
            if let Ok(path_str) = core::str::from_utf8(&path_buf) {
                if vfs_unlink(path_str).is_ok() {
                    return 0;
                }
            }
            u64::MAX
        }
        SYS_SPAWN => {
            let path_ptr = arg1 as *const u8;
            let path_len = arg2 as usize;
            if let Err(_) = validate_user_buffer(arg1, path_len) {
                return u64::MAX;
            }
            let mut path_buf = alloc::vec![0u8; path_len];
            if copy_from_user(path_ptr, &mut path_buf).is_err() {
                return u64::MAX;
            }
            if let Ok(path_str) = core::str::from_utf8(&path_buf) {
                if let Ok(file) = vfs_open(path_str, 0) {
                    let mut elf_data = alloc::vec![0u8; 65536];
                    if let Ok(n) = file.read(0, &mut elf_data) {
                        if let Ok(loaded) = crate::task::elf::load_elf_into_address_space(&elf_data[..n]) {
                            let task = crate::task::process::Task::new_user_task("user_proc", loaded, 1);
                            let pid = task.process.pid;
                            let mut sched = crate::task::scheduler::SCHEDULER.lock();
                            if sched.add_task(task).is_ok() {
                                return pid;
                            }
                        }
                    }
                }
            }
            u64::MAX
        }
        SYS_KILL => {
            let target_pid = arg1;
            let mut sched = crate::task::scheduler::SCHEDULER.lock();
            for task in sched.tasks.iter_mut() {
                if task.process.pid == target_pid {
                    task.state = crate::task::process::TaskState::Dead;
                    return 0;
                }
            }
            u64::MAX
        }
        SYS_WAIT => {
            let target_pid = arg1;
            let sched = crate::task::scheduler::SCHEDULER.lock();
            let is_alive = sched.is_task_alive(target_pid);
            drop(sched);
            if is_alive {
                crate::task::scheduler::do_schedule();
                0
            } else {
                0
            }
        }
        SYS_GETCWD => {
            let ptr = arg1 as *mut u8;
            let len = arg2 as usize;
            if let Err(_) = validate_user_buffer(arg1, len) {
                return u64::MAX;
            }
            let cwd_bytes = b"/";
            if len < cwd_bytes.len() {
                return u64::MAX;
            }
            if copy_to_user(cwd_bytes, ptr).is_ok() {
                return cwd_bytes.len() as u64;
            }
            u64::MAX
        }
        SYS_CHDIR => {
            let path_ptr = arg1 as *const u8;
            let path_len = arg2 as usize;
            if let Err(_) = validate_user_buffer(arg1, path_len) {
                return u64::MAX;
            }
            let mut path_buf = alloc::vec![0u8; path_len];
            if copy_from_user(path_ptr, &mut path_buf).is_err() {
                return u64::MAX;
            }
            if let Ok(path_str) = core::str::from_utf8(&path_buf) {
                if vfs_open(path_str, 0).is_ok() {
                    return 0;
                }
            }
            u64::MAX
        }
        _ => {
            crate::kprintln!("[SYSCALL] Unknown syscall: {}", sys_no);
            u64::MAX
        }
    }
}
