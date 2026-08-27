#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
pub struct SimpleTextOutput {
    reset: usize,
    pub output_string: unsafe extern "win64" fn(this: *mut SimpleTextOutput, string: *const u16) -> usize,
    test_string: usize,
    query_mode: usize,
    set_mode: usize,
    set_attribute: usize,
    clear_screen: unsafe extern "win64" fn(this: *mut SimpleTextOutput) -> usize,
}

#[repr(C)]
pub struct SystemTable {
    header: [u64; 3],
    firmware_vendor: usize,
    firmware_revision: u32,
    console_in_handle: usize,
    con_in: usize,
    console_out_handle: usize,
    pub con_out: *mut SimpleTextOutput,
}

#[no_mangle]
pub unsafe extern "win64" fn efi_main(_image_handle: usize, system_table: *mut SystemTable) -> usize {
    if !system_table.is_null() && !(*system_table).con_out.is_null() {
        let con_out = (*system_table).con_out;
        
        // UTF-16 message: "CatalystOS UEFI Loader Online!\r\n"
        let msg: [u16; 34] = [
            'C' as u16, 'a' as u16, 't' as u16, 'a' as u16, 'l' as u16, 'y' as u16, 's' as u16, 't' as u16,
            'O' as u16, 'S' as u16, ' ' as u16, 'U' as u16, 'E' as u16, 'F' as u16, 'I' as u16, ' ' as u16,
            'L' as u16, 'o' as u16, 'a' as u16, 'd' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'O' as u16,
            'n' as u16, 'l' as u16, 'i' as u16, 'n' as u16, 'e' as u16, '!' as u16, '\r' as u16, '\n' as u16,
            0, 0
        ];
        
        ((*con_out).output_string)(con_out, msg.as_ptr());
    }
    
    loop {}
}
