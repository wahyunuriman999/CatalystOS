// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const STACK_SIZE: usize = 4096 * 5;

#[repr(align(16))]
#[repr(align(4096))]
#[repr(align(16))]
struct Stack([u8; STACK_SIZE]);

static mut DOUBLE_FAULT_STACK: Stack = Stack([0; STACK_SIZE]);
static mut PRIVILEGE_STACK: Stack = Stack([0; STACK_SIZE]);

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        
        let df_stack_start = VirtAddr::from_ptr(&raw const DOUBLE_FAULT_STACK);
        let priv_stack_start = VirtAddr::from_ptr(&raw const PRIVILEGE_STACK);
        
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = df_stack_start + STACK_SIZE as u64;
        tss.privilege_stack_table[0] = priv_stack_start + STACK_SIZE as u64;
        
        crate::kprintln!("DF Stack Base: {:#x}", df_stack_start.as_u64());
        crate::kprintln!("DF Stack End (IST0): {:#x}", tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize].as_u64());
        crate::kprintln!("Priv Stack Base: {:#x}", priv_stack_start.as_u64());
        crate::kprintln!("Priv Stack End (RSP0): {:#x}", tss.privilege_stack_table[0].as_u64());

        tss
    };
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());
        
        // SYSRET expects:
        // STAR[63:48] = user_code_32
        // STAR[63:48] + 8 = user_data
        // STAR[63:48] + 16 = user_code_64
        let user_code_32_selector = gdt.append(Descriptor::user_data_segment()); // Dummy 32-bit code
        let user_data_selector = gdt.append(Descriptor::user_data_segment());
        let user_code_selector = gdt.append(Descriptor::user_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        
        let tss_ptr = &*TSS as *const _ as u64;
        crate::kprintln!("TSS Base: {:#x}", tss_ptr);

        (gdt, Selectors {
            kernel_code_selector,
            kernel_data_selector,
            user_code_32_selector,
            user_code_selector,
            user_data_selector,
            tss_selector,
        })
    };
}

#[derive(Clone)]
pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_32_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub fn init() {
    GDT.0.load();
    unsafe {
        use x86_64::instructions::segmentation::{CS, DS, ES, FS, GS, SS, Segment};
        CS::set_reg(GDT.1.kernel_code_selector);
        DS::set_reg(GDT.1.kernel_data_selector);
        ES::set_reg(GDT.1.kernel_data_selector);
        SS::set_reg(GDT.1.kernel_data_selector);
        
        x86_64::instructions::tables::load_tss(GDT.1.tss_selector);
    }
}

pub fn get_selectors() -> Selectors {
    GDT.1.clone()
}

pub unsafe fn set_rsp0(rsp: u64) {
    let tss_ptr = &*TSS as *const _ as *mut TaskStateSegment;
    (*tss_ptr).privilege_stack_table[0] = VirtAddr::new(rsp);
}
