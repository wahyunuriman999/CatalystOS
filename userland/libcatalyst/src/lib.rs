// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#![no_std]

use core::arch::asm;
use core::fmt::{self, Write};

pub const SYS_EXIT: u64          = 1;
pub const SYS_YIELD: u64         = 2;
pub const SYS_GETPID: u64        = 4;
pub const SYS_OPEN: u64          = 10;
pub const SYS_CLOSE: u64         = 11;
pub const SYS_READ: u64          = 12;
pub const SYS_WRITE: u64         = 13;
pub const SYS_MKDIR: u64         = 16;
pub const SYS_UNLINK: u64        = 17;
pub const SYS_SPAWN: u64         = 30;
pub const SYS_WAIT: u64          = 31;

pub const O_RDONLY: u32 = 0;
pub const O_WRONLY: u32 = 1;
pub const O_RDWR: u32   = 2;
pub const O_CREAT: u32  = 0x40;
pub const O_TRUNC: u32  = 0x200;

#[inline(always)]
pub unsafe fn syscall3(no: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let ret: u64;
    asm!(
        "syscall",
        inlateout("rax") no => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        out("rcx") _,
        out("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall2(no: u64, arg1: u64, arg2: u64) -> u64 {
    syscall3(no, arg1, arg2, 0)
}

#[inline(always)]
pub unsafe fn syscall1(no: u64, arg1: u64) -> u64 {
    syscall3(no, arg1, 0, 0)
}

#[inline(always)]
pub unsafe fn syscall0(no: u64) -> u64 {
    syscall3(no, 0, 0, 0)
}

pub fn exit(code: u64) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code);
    }
    loop {}
}

pub fn yield_now() {
    unsafe {
        syscall0(SYS_YIELD);
    }
}

pub fn getpid() -> u64 {
    unsafe {
        syscall0(SYS_GETPID)
    }
}

pub fn write_fd(fd: usize, buf: &[u8]) -> usize {
    unsafe {
        syscall3(SYS_WRITE, fd as u64, buf.as_ptr() as u64, buf.len() as u64) as usize
    }
}

pub fn read_fd(fd: usize, buf: &mut [u8]) -> usize {
    unsafe {
        syscall3(SYS_READ, fd as u64, buf.as_mut_ptr() as u64, buf.len() as u64) as usize
    }
}

pub fn open(path: &str, flags: u32) -> Result<usize, ()> {
    let ret = unsafe {
        syscall3(SYS_OPEN, path.as_ptr() as u64, path.len() as u64, flags as u64)
    };
    if ret == u64::MAX {
        Err(())
    } else {
        Ok(ret as usize)
    }
}

pub fn close(fd: usize) -> Result<(), ()> {
    let ret = unsafe {
        syscall1(SYS_CLOSE, fd as u64)
    };
    if ret == 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub fn mkdir(path: &str) -> Result<(), ()> {
    let ret = unsafe {
        syscall2(SYS_MKDIR, path.as_ptr() as u64, path.len() as u64)
    };
    if ret == 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub fn unlink(path: &str) -> Result<(), ()> {
    let ret = unsafe {
        syscall2(SYS_UNLINK, path.as_ptr() as u64, path.len() as u64)
    };
    if ret == 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub fn spawn(path: &str) -> Result<u64, ()> {
    let ret = unsafe {
        syscall2(SYS_SPAWN, path.as_ptr() as u64, path.len() as u64)
    };
    if ret == u64::MAX {
        Err(())
    } else {
        Ok(ret)
    }
}

pub fn wait(pid: u64) -> u64 {
    unsafe {
        syscall1(SYS_WAIT, pid)
    }
}

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_fd(1, s.as_bytes());
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let mut stdout = Stdout;
    stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
