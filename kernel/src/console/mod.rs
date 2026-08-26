use core::fmt::{self, Write};
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;

pub mod vga;

pub static SERIAL: Mutex<Option<SerialPort>> = Mutex::new(None);

pub fn init() {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    *SERIAL.lock() = Some(serial_port);
}

pub fn init_vga(framebuffer: &'static mut [u8], info: bootloader_api::info::FrameBufferInfo) {
    vga::init(framebuffer, info);

    let gui_buf = unsafe {
        core::slice::from_raw_parts_mut(
            vga::FRAMEBUFFER_WRITER.lock()
                .as_ref()
                .map(|w| w.buffer_ptr())
                .unwrap_or(core::ptr::null_mut()),
            info.byte_len,
        )
    };
    if !gui_buf.as_ptr().is_null() {
        crate::graphics::canvas::store_framebuffer(gui_buf, &info);
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        let debugcon = unsafe { x86_64::instructions::port::PortWriteOnly::<u8>::new(0xe9) };
        let mut writer = DebugWriter { port: debugcon };
        let _ = core::fmt::write(&mut writer, args);
        
        let mut serial = SERIAL.lock();
        if let Some(ref mut port) = *serial {
            let _ = port.write_fmt(args);
        }
        
        let mut vga = vga::FRAMEBUFFER_WRITER.lock();
        if let Some(ref mut writer) = *vga {
            let _ = writer.write_fmt(args);
        }
    });
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

pub fn shutdown() -> ! {
    for _ in 0..50_000_000 { unsafe { core::arch::asm!("nop") } }
    unsafe {
        let mut port = x86_64::instructions::port::PortWriteOnly::new(0xf4);
        port.write(0_u32);
    }
    loop { x86_64::instructions::hlt(); }
}

struct DebugWriter {
    port: x86_64::instructions::port::PortWriteOnly<u8>,
}
impl core::fmt::Write for DebugWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            unsafe { self.port.write(byte); }
        }
        Ok(())
    }
}
