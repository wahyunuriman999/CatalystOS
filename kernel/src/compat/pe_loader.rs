use alloc::vec::Vec;
use crate::compat::win32;
use x86_64::structures::paging::{Page, Size4KiB, PageTableFlags, Mapper, FrameAllocator};
use x86_64::VirtAddr;

const MZ_MAGIC: u16 = 0x5A4D; // 'MZ'
const PE_SIGNATURE: u32 = 0x00004550; // 'PE\0\0'
const IMAGE_NT_OPTIONAL_HDR64_MAGIC: u16 = 0x020B; // PE32+

#[derive(Debug)]
pub enum PeError {
    InvalidMagic, InvalidPe, TruncatedHeader, UnsupportedFormat, LoadError(&'static str),
}

pub struct LoadedPe {
    pub entry_point: u64,
    pub image_base: u64,
    pub size: usize,
}

// static mut STUB_ARENA removed
static mut STUB_OFFSET: usize = 0;

pub fn load_pe_into_memory(data: &[u8], mapper: &mut impl Mapper<Size4KiB>) -> Result<LoadedPe, PeError> {
    if data.len() < 64 { return Err(PeError::TruncatedHeader); }
    let magic = u16::from_le_bytes([data[0], data[1]]);
    if magic != MZ_MAGIC { return Err(PeError::InvalidMagic); }
    let e_lfanew = i32::from_le_bytes([data[60], data[61], data[62], data[63]]) as usize;
    if e_lfanew + 24 > data.len() { return Err(PeError::TruncatedHeader); }
    let pe_sig = u32::from_le_bytes([data[e_lfanew], data[e_lfanew+1], data[e_lfanew+2], data[e_lfanew+3]]);
    if pe_sig != PE_SIGNATURE { return Err(PeError::InvalidPe); }
    
    let coff_base = e_lfanew + 4;
    let num_sections = u16::from_le_bytes([data[coff_base+2], data[coff_base+3]]) as usize;
    let opt_header_size = u16::from_le_bytes([data[coff_base+16], data[coff_base+17]]) as usize;
    
    let opt_base = coff_base + 20;
    if opt_base + 24 > data.len() { return Err(PeError::TruncatedHeader); }
    let opt_magic = u16::from_le_bytes([data[opt_base], data[opt_base+1]]);
    if opt_magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC { return Err(PeError::UnsupportedFormat); }
    
    let entry_point_rva = u32::from_le_bytes([data[opt_base+16], data[opt_base+17], data[opt_base+18], data[opt_base+19]]);
    let image_size = u32::from_le_bytes([data[opt_base+56], data[opt_base+57], data[opt_base+58], data[opt_base+59]]) as usize;
    let hdr_size = u32::from_le_bytes([data[opt_base+60], data[opt_base+61], data[opt_base+62], data[opt_base+63]]) as usize;
    
    // Allocate memory for the image at user space 0x2000_0000_0000
    let user_base: u64 = 0x2000_0000_0000;
    if image_size == 0 || image_size > 0x10000000 {
        return Err(PeError::LoadError("Invalid image size"));
    }

    {
        // 1. Allocate pages
        let mut frame_allocator = crate::memory::frame_allocator::FRAME_ALLOCATOR.lock();
        let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(user_base));
        let num_pages = (image_size + 4095) / 4096;
        for i in 0..num_pages {
            let page = start_page + i as u64;
            if let Some(frame) = frame_allocator.allocate_frame() {
                unsafe {
                    mapper.map_to(page, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE, &mut *frame_allocator)
                        .expect("Map failed").flush();
                }
            } else {
                return Err(PeError::LoadError("OOM"));
            }
        }
    }
    
    // Stub pages
    {
        let mut frame_allocator = crate::memory::frame_allocator::FRAME_ALLOCATOR.lock();
        let stub_base: u64 = 0x2000_1000_0000;
        let stub_page = Page::<Size4KiB>::containing_address(VirtAddr::new(stub_base));
        if let Some(frame) = frame_allocator.allocate_frame() {
            unsafe {
                mapper.map_to(stub_page, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE, &mut *frame_allocator)
                    .expect("Map stub failed").flush();
            }
        }
    }
    
    unsafe { STUB_OFFSET = 0; }
    
    let mem_ptr = user_base as *mut u8;

    let hdr_copy_len = core::cmp::min(hdr_size as usize, core::cmp::min(data.len(), image_size as usize));
    unsafe { core::ptr::copy_nonoverlapping(data.as_ptr(), mem_ptr, hdr_copy_len); }
    
    let section_table_offset = opt_base + opt_header_size;
    
    for i in 0..num_sections {
        let sec_off = section_table_offset + i * 40;
        if sec_off + 40 > data.len() { break; }
        
        let virtual_address = u32::from_le_bytes([data[sec_off+12], data[sec_off+13], data[sec_off+14], data[sec_off+15]]);
        let raw_data_size = u32::from_le_bytes([data[sec_off+16], data[sec_off+17], data[sec_off+18], data[sec_off+19]]);
        let raw_data_ptr = u32::from_le_bytes([data[sec_off+20], data[sec_off+21], data[sec_off+22], data[sec_off+23]]);
        
        if raw_data_size > 0 && raw_data_ptr > 0 {
            let src_start = raw_data_ptr as usize;
            let src_end = core::cmp::min(src_start + raw_data_size as usize, data.len());
            if src_start < data.len() {
                let copy_len = src_end - src_start;
                // Add bounds check for virtual_address
                if (virtual_address as usize) + copy_len <= image_size as usize {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            data.as_ptr().add(src_start),
                            mem_ptr.add(virtual_address as usize),
                            copy_len
                        );
                    }
                } else {
                    crate::kprintln!("[PE] WARNING: Section out of bounds, skipping copy");
                }
            }
        }
    }
    
    if opt_base + 124 <= data.len() {
        let import_dir_rva = u32::from_le_bytes([data[opt_base+120], data[opt_base+121], data[opt_base+122], data[opt_base+123]]);
        if import_dir_rva != 0 && (import_dir_rva as usize) + 20 <= image_size {
            resolve_imports(mem_ptr, import_dir_rva, image_size as u32);
        }
    }
    
    Ok(LoadedPe {
        entry_point: user_base + entry_point_rva as u64,
        image_base: user_base,
        size: image_size,
    })
}

fn resolve_imports(base: *mut u8, import_dir_rva: u32, image_size: u32) {
    let mut current_rva = import_dir_rva as usize;
    loop {
        if current_rva + 20 > image_size as usize { break; }
        let import_desc = unsafe { base.add(current_rva) };
        let orig_first_thunk = unsafe { core::ptr::read_unaligned(import_desc as *const u32) };
        let name_rva = unsafe { core::ptr::read_unaligned(import_desc.add(12) as *const u32) };
        let first_thunk = unsafe { core::ptr::read_unaligned(import_desc.add(16) as *const u32) };
        
        if orig_first_thunk == 0 && first_thunk == 0 { break; }
        
        if name_rva as usize >= image_size as usize {
            current_rva += 20;
            continue;
        }
        
        let dll_name_ptr = unsafe { base.add(name_rva as usize) };
        let mut dll_name_len = 0;
        while (name_rva as usize + dll_name_len) < image_size as usize && unsafe { *dll_name_ptr.add(dll_name_len) } != 0 { dll_name_len += 1; }
        let dll_name_slice = unsafe { core::slice::from_raw_parts(dll_name_ptr, dll_name_len) };
        let dll_name = core::str::from_utf8(dll_name_slice).unwrap_or("unknown").trim_end_matches(".dll").trim_end_matches(".DLL");
        
        let mut thunk_rva = if orig_first_thunk != 0 { orig_first_thunk } else { first_thunk };
        let mut iat_rva = first_thunk;
        
        loop {
            if thunk_rva as usize + 8 > image_size as usize || iat_rva as usize + 8 > image_size as usize { break; }
            let thunk_data = unsafe { core::ptr::read_unaligned(base.add(thunk_rva as usize) as *const u64) };
            if thunk_data == 0 { break; }
            
            if thunk_data & (1 << 63) == 0 {
                let name_data_rva = (thunk_data & 0x7FFF_FFFF) as usize;
                if name_data_rva + 2 >= image_size as usize { break; }
                let func_name_ptr = unsafe { base.add(name_data_rva + 2) };
                let mut func_name_len = 0;
                while (name_data_rva + 2 + func_name_len) < image_size as usize && unsafe { *func_name_ptr.add(func_name_len) } != 0 { func_name_len += 1; }
                let func_name_slice = unsafe { core::slice::from_raw_parts(func_name_ptr, func_name_len) };
                let func_name = core::str::from_utf8(func_name_slice).unwrap_or("unknown");
                crate::kprintln!("[PE] Importing: {}!{}", dll_name, func_name);
                
                let stub_addr = create_stub(dll_name, func_name);
                unsafe { core::ptr::write_unaligned(base.add(iat_rva as usize) as *mut u64, stub_addr); }
            }
            
            thunk_rva += 8;
            iat_rva += 8;
        }
        
        current_rva += 20;
    }
}

fn create_stub(_dll: &str, func: &str) -> u64 {
    let syscall_id: u32 = match func {
        "WriteConsoleA" => 101,
        "GetStdHandle" => 102,
        "ExitProcess" => 103,
        _ => 999,
    };
    
    unsafe {
        if STUB_OFFSET >= 4000 {
            crate::kprintln!("[PE] WARNING: Stub arena exhausted");
            return 0; // Return null stub if out of memory
        }
        let addr = 0x2000_1000_0000 + STUB_OFFSET as u64;
        
        // mov r10, rcx (49 89 CA)
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET) as *mut u8) = 0x49; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 1) as *mut u8) = 0x89; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 2) as *mut u8) = 0xCA; }
        
        // mov eax, syscall_id (B8 id id id id)
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 3) as *mut u8) = 0xB8; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 4) as *mut u8) = (syscall_id & 0xFF) as u8; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 5) as *mut u8) = ((syscall_id >> 8) & 0xFF) as u8; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 6) as *mut u8) = ((syscall_id >> 16) & 0xFF) as u8; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 7) as *mut u8) = ((syscall_id >> 24) & 0xFF) as u8; }
        
        // syscall (0F 05)
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 8) as *mut u8) = 0x0F; }
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 9) as *mut u8) = 0x05; }
        
        // ret (C3)
        unsafe { *((0x2000_1000_0000 + STUB_OFFSET + 10) as *mut u8) = 0xC3; }
        
        STUB_OFFSET += 16;
        addr
    }
}
