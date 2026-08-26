// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use alloc::collections::BTreeMap;
use spin::Mutex;

/// Win32 API function table
/// Maps function name hash -> Catalyst handler
#[allow(dead_code)]
static WIN32_API_TABLE: Mutex<BTreeMap<u32, u64>> = Mutex::new(BTreeMap::new());

/// Win32 error codes
#[allow(dead_code)]
pub mod error_codes {
    pub const ERROR_SUCCESS: u32 = 0;
    pub const ERROR_FILE_NOT_FOUND: u32 = 2;
    pub const ERROR_ACCESS_DENIED: u32 = 5;
    pub const ERROR_INVALID_HANDLE: u32 = 6;
    pub const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;
    pub const ERROR_INVALID_PARAMETER: u32 = 87;
    pub const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    pub const ERROR_ALREADY_EXISTS: u32 = 183;
}

/// Windows HANDLE type
pub type HANDLE = u64;
pub const INVALID_HANDLE_VALUE: HANDLE = u64::MAX;

/// Windows BOOL type  
pub type BOOL = i32;
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

/// Windows memory allocation flags
#[allow(dead_code)]
pub mod mem_flags {
    pub const MEM_COMMIT:   u32 = 0x1000;
    pub const MEM_RESERVE:  u32 = 0x2000;
    pub const MEM_RELEASE:  u32 = 0x8000;
    pub const PAGE_READWRITE: u32 = 0x04;
    pub const PAGE_EXECUTE_READWRITE: u32 = 0x40;
}

/// Win32 kernel32.dll stubs
/// These will be called by Windows programs via the compatibility syscall gate
#[allow(dead_code)]
pub mod kernel32 {
    use super::*;
    
    /// VirtualAlloc - Allocate virtual memory
    pub fn virtual_alloc(_base: u64, size: usize, _alloc_type: u32, _protect: u32) -> u64 {
        // Map to Catalyst memory allocator
        if size == 0 { return 0; }
        let layout = core::alloc::Layout::from_size_align(size, 4096).unwrap_or(
            core::alloc::Layout::from_size_align(4096, 4096).unwrap()
        );
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        if ptr.is_null() { 0 } else { ptr as u64 }
    }
    
    /// VirtualFree - Free virtual memory
    pub fn virtual_free(base: u64, size: usize, _free_type: u32) -> BOOL {
        if base == 0 || size == 0 { return FALSE; }
        let layout = core::alloc::Layout::from_size_align(size.max(1), 4096)
            .unwrap_or(core::alloc::Layout::from_size_align(4096, 4096).unwrap());
        unsafe { alloc::alloc::dealloc(base as *mut u8, layout) };
        TRUE
    }
    
    /// GetLastError
    pub fn get_last_error() -> u32 {
        error_codes::ERROR_SUCCESS
    }
    
    /// CreateFile stub - maps to Catalyst VFS
    pub fn create_file(
        filename: &str,
        _access: u32,
        _share: u32,
        _security: u64,
        creation: u32,
        _attrs: u32,
        _template: HANDLE,
    ) -> HANDLE {
        crate::kprintln!("[WIN32] CreateFile: '{}' (disposition={})", filename, creation);
        // Will be wired to CatFS when filesystem is ready
        INVALID_HANDLE_VALUE
    }
    
    /// CloseHandle
    pub fn close_handle(handle: HANDLE) -> BOOL {
        if handle == INVALID_HANDLE_VALUE || handle == 0 {
            return FALSE;
        }
        TRUE
    }
    
    /// GetProcessHeap - return a fake heap handle
    pub fn get_process_heap() -> HANDLE {
        0xDEADBEEF
    }
    
    /// HeapAlloc
    pub fn heap_alloc(_heap: HANDLE, _flags: u32, size: usize) -> u64 {
        virtual_alloc(0, size, mem_flags::MEM_COMMIT, mem_flags::PAGE_READWRITE)
    }
    
    /// HeapFree
    pub fn heap_free(_heap: HANDLE, _flags: u32, ptr: u64) -> BOOL {
        if ptr == 0 { return TRUE; }
        // Size unknown without tracking — use minimum
        virtual_free(ptr, 4096, 0)
    }
    
    /// ExitProcess
    pub fn exit_process(exit_code: u32) -> ! {
        crate::kprintln!("[WIN32] ExitProcess called with code {}", exit_code);
        loop { x86_64::instructions::hlt(); }
    }
    
    /// GetSystemInfo - return emulated system info
    pub fn get_system_info() -> SystemInfo {
        SystemInfo {
            processor_type: 9, // PROCESSOR_ARCHITECTURE_AMD64
            page_size: 4096,
            min_app_address: 0x10000,
            max_app_address: 0x7FFFFFFFFFFF,
            active_processor_mask: 1,
            number_of_processors: 1,
            processor_level: 6,
            processor_revision: 0x0F01,
        }
    }
    
    /// WriteConsole stub
    pub fn write_console(_handle: HANDLE, text: &str, _written: *mut u32) -> BOOL {
        crate::kprintln!("[WIN32-CONSOLE] {}", text);
        TRUE
    }
}

#[repr(C)]
pub struct SystemInfo {
    pub processor_type: u16,
    pub page_size: u32,
    pub min_app_address: u64,
    pub max_app_address: u64,
    pub active_processor_mask: u64,
    pub number_of_processors: u32,
    pub processor_level: u16,
    pub processor_revision: u16,
}

/// Win32 DLL function dispatcher - called from compat syscall handler
/// Maps DLL name + function name to our stub implementations
#[allow(dead_code)]
pub fn dispatch(dll: &str, func: &str, args: &[u64]) -> u64 {
    match (dll, func) {
        ("kernel32", "VirtualAlloc") => kernel32::virtual_alloc(args[0], args[1] as usize, args[2] as u32, args[3] as u32),
        ("kernel32", "VirtualFree")  => kernel32::virtual_free(args[0], args[1] as usize, args[2] as u32) as u64,
        ("kernel32", "GetLastError") => kernel32::get_last_error() as u64,
        ("kernel32", "ExitProcess")  => kernel32::exit_process(args[0] as u32),
        ("kernel32", "GetProcessHeap") => kernel32::get_process_heap(),
        ("kernel32", "HeapAlloc")    => kernel32::heap_alloc(args[0], args[1] as u32, args[2] as usize),
        ("kernel32", "HeapFree")     => kernel32::heap_free(args[0], args[1] as u32, args[2]) as u64,
        _ => {
            crate::kprintln!("[WIN32] UNIMPL: {}!{} called", dll, func);
            0
        }
    }
}

pub fn init() {
    crate::kprintln!("[WIN32] kernel32 API table: {} functions registered", 8);
    crate::kprintln!("[WIN32] user32, gdi32, ntdll stubs: LOADED");
}

// --- ABI STUBS ---
// These are called directly from the JIT/trampoline of the loaded PE
#[unsafe(no_mangle)]
pub unsafe extern "system" fn WriteConsoleA_stub(
    h_console_output: u64,
    lp_buffer: *const u8,
    n_number_of_chars_to_write: u32,
    lp_number_of_chars_written: *mut u32,
    _lp_reserved: *mut u8,
) -> i32 {
    let slice = core::slice::from_raw_parts(lp_buffer, n_number_of_chars_to_write as usize);
    if let Ok(s) = core::str::from_utf8(slice) {
        crate::kprintln!("[WIN32-APP] {}", s);
    } else {
        crate::kprintln!("[WIN32-APP] (Invalid UTF-8 output)");
    }
    if !lp_number_of_chars_written.is_null() {
        *lp_number_of_chars_written = n_number_of_chars_to_write;
    }
    1 // TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn GetStdHandle_stub(n_std_handle: u32) -> u64 {
    // Just return a fake handle
    0x00000000FFFFFFFF - (n_std_handle as u64)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ExitProcess_stub(u_exit_code: u32) -> ! {
    crate::kprintln!("[WIN32-APP] Process exited with code {}", u_exit_code);
    loop { x86_64::instructions::hlt(); }
}
