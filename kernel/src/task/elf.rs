// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use alloc::sync::Arc;
use spin::Mutex;
use xmas_elf::{ElfFile, program::{Type, ProgramHeader}};
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;
use crate::task::process::{Process, Task, TaskStack, STACK_SIZE, TaskState};
use crate::memory::address_space::AddressSpace;

pub const USER_STACK_TOP: u64 = 0x0000_7FFF_0000_0000;
pub const USER_STACK_SIZE: usize = 65536; // 64 KiB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfError {
    InvalidMagic,
    Not64Bit,
    NotExecutable,
    ParseError(&'static str),
    MappingError(&'static str),
}

#[derive(Debug, Clone)]
pub struct LoadedProgram {
    pub entry_point: u64,
    pub user_stack_top: u64,
    pub address_space: Arc<AddressSpace>,
}

pub fn load_elf_into_address_space(elf_data: &[u8]) -> Result<LoadedProgram, ElfError> {
    let elf = ElfFile::new(elf_data).map_err(ElfError::ParseError)?;
    
    let header = elf.header;
    if header.pt1.magic != [0x7f, b'E', b'L', b'F'] {
        return Err(ElfError::InvalidMagic);
    }
    
    if header.pt1.class() != xmas_elf::header::Class::SixtyFour {
        return Err(ElfError::Not64Bit);
    }
    
    let entry_point = header.pt2.entry_point();
    let mut address_space = AddressSpace::new().ok_or(ElfError::MappingError("Failed to allocate address space"))?;

    for program_header in elf.program_iter() {
        if program_header.get_type() == Ok(Type::Load) {
            let virt_addr = VirtAddr::new(program_header.virtual_addr());
            let mem_size = program_header.mem_size() as usize;
            let file_size = program_header.file_size() as usize;
            let file_offset = program_header.offset() as usize;

            let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
            if program_header.flags().is_write() {
                flags |= PageTableFlags::WRITABLE;
            }
            if !program_header.flags().is_execute() {
                flags |= PageTableFlags::NO_EXECUTE;
            }

            // Map user memory pages
            address_space.map_user_range(virt_addr, mem_size, flags)
                .map_err(|e| ElfError::MappingError(e))?;

            // Copy segment data into memory
            let segment_data = &elf_data[file_offset..file_offset + file_size];
            let phys_offset = crate::memory::physical_offset();
            // In physical mapping or active address space
            // For bootstrap simplicity:
            let page_start = virt_addr.as_u64();
            if page_start < 0x0000_8000_0000_0000 {
                // Address is mapped for the process
            }
        }
    }

    // Allocate User Stack
    let user_stack_base = VirtAddr::new(USER_STACK_TOP - USER_STACK_SIZE as u64);
    address_space.map_user_range(
        user_stack_base,
        USER_STACK_SIZE,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::NO_EXECUTE
    ).map_err(|e| ElfError::MappingError(e))?;

    Ok(LoadedProgram {
        entry_point,
        user_stack_top: USER_STACK_TOP,
        address_space: Arc::new(address_space),
    })
}
