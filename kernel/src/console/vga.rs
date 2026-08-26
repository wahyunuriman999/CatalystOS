use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::{fmt, ptr};
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};
use spin::Mutex;

pub static FRAMEBUFFER_WRITER: Mutex<Option<FramebufferWriter>> = Mutex::new(None);

const LINE_SPACING: usize = 2;
const LETTER_SPACING: usize = 0;

pub struct FramebufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    term_x: usize,
    term_y: usize,
    term_w: usize,
    term_h: usize,
    x_pos: usize,
    y_pos: usize,
    pub disabled: bool,
}

impl FramebufferWriter {
    pub fn clear(&mut self) {
        for byte in self.framebuffer.iter_mut() {
            *byte = 0;
        }
        self.x_pos = 0;
        self.y_pos = 0;
    }

    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut writer = Self {
            framebuffer,
            info,
            term_x: 0,
            term_y: 0,
            term_w: info.width,
            term_h: info.height,
            x_pos: 0,
            y_pos: 0,
            disabled: false,
        };
        writer.draw_gui();
        writer
    }

    pub fn draw_gui(&mut self) {
        // Desktop Background (Dark Blue-ish)
        self.fill_rect(0, 0, self.info.width, self.info.height, [15, 20, 35]);
        
        // Taskbar (Bottom)
        let taskbar_h = 40;
        let taskbar_y = self.info.height.saturating_sub(taskbar_h);
        self.fill_rect(0, taskbar_y, self.info.width, taskbar_h, [30, 35, 50]);
        
        // Taskbar Start Button (Catalyst Logo placeholder)
        self.fill_rect(10, taskbar_y + 5, 30, 30, [0, 150, 255]);
        self.draw_string_abs(50, taskbar_y + 12, "Catalyst OS", [255, 255, 255]);

        // Terminal Window
        let win_w = 640.min(self.info.width.saturating_sub(40));
        let win_h = 400.min(self.info.height.saturating_sub(100));
        let win_x = (self.info.width - win_w) / 2;
        let win_y = (self.info.height - taskbar_h - win_h) / 2;

        let title_h = 30;
        
        // Window shadow/border
        self.fill_rect(win_x-2, win_y-2, win_w+4, win_h+4, [50, 50, 50]);
        
        // Title bar
        self.fill_rect(win_x, win_y, win_w, title_h, [40, 45, 60]);
        self.draw_string_abs(win_x + 10, win_y + 8, "Terminal - root@catalyst", [200, 200, 200]);
        
        // Window close button (red)
        self.fill_rect(win_x + win_w - 30, win_y + 5, 20, 20, [200, 50, 50]);

        // Terminal Background
        let term_y = win_y + title_h;
        let term_h = win_h - title_h;
        self.fill_rect(win_x, term_y, win_w, term_h, [10, 10, 10]);

        // Set terminal bounds for future text output
        self.term_x = win_x + 5;
        self.term_y = term_y + 5;
        self.term_w = win_w - 10;
        self.term_h = term_h - 10;
        self.x_pos = 0;
        self.y_pos = 0;
    }

    fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: [u8; 3]) {
        for cy in y..y+h {
            for cx in x..x+w {
                self.write_pixel(cx, cy, color);
            }
        }
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }
        let byte_offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        if byte_offset + 2 < self.framebuffer.len() {
            match self.info.pixel_format {
                PixelFormat::Rgb => {
                    self.framebuffer[byte_offset] = color[0];
                    self.framebuffer[byte_offset + 1] = color[1];
                    self.framebuffer[byte_offset + 2] = color[2];
                }
                PixelFormat::Bgr => {
                    self.framebuffer[byte_offset] = color[2];
                    self.framebuffer[byte_offset + 1] = color[1];
                    self.framebuffer[byte_offset + 2] = color[0];
                }
                PixelFormat::U8 => {
                    let gray = (color[0] as u16 + color[1] as u16 + color[2] as u16) / 3;
                    self.framebuffer[byte_offset] = gray as u8;
                }
                _ => {}
            }
        }
    }
    
    // Absolute position string drawing (for GUI elements)
    pub fn draw_string_abs(&mut self, mut x: usize, y: usize, s: &str, color: [u8; 3]) {
        for c in s.chars() {
            let raster = get_raster(c, FontWeight::Regular, RasterHeight::Size16)
                .unwrap_or_else(|| get_raster('?', FontWeight::Regular, RasterHeight::Size16).unwrap());
            
            for (ry, row) in raster.raster().iter().enumerate() {
                for (rx, byte) in row.iter().enumerate() {
                    if *byte > 0 {
                        // Blend text color based on intensity (basic threshold for now)
                        self.write_pixel(x + rx, y + ry, color);
                    }
                }
            }
            x += raster.width() + LETTER_SPACING;
        }
    }

    fn newline(&mut self) {
        self.y_pos += 16 + LINE_SPACING;
        self.x_pos = 0;
        if self.y_pos + 16 > self.term_h {
            self.scroll_terminal();
        }
    }
    
    fn scroll_terminal(&mut self) {
        // Simple scroll: just clear the terminal area for now
        self.fill_rect(self.term_x, self.term_y, self.term_w, self.term_h, [10, 10, 10]);
        self.y_pos = 0;
        self.x_pos = 0;
    }

    pub fn write_char(&mut self, c: char) {
        if self.disabled { return; }
        if c == '\n' {
            self.newline();
            return;
        }
        if c == '\r' {
            self.x_pos = 0;
            return;
        }

        let raster = get_raster(c, FontWeight::Regular, RasterHeight::Size16)
            .unwrap_or_else(|| get_raster('?', FontWeight::Regular, RasterHeight::Size16).unwrap());

        if self.x_pos + raster.width() > self.term_w {
            self.newline();
        }

        for (ry, row) in raster.raster().iter().enumerate() {
            for (rx, byte) in row.iter().enumerate() {
                if *byte > 0 {
                    self.write_pixel(self.term_x + self.x_pos + rx, self.term_y + self.y_pos + ry, [200, 200, 200]);
                }
            }
        }
        self.x_pos += raster.width() + LETTER_SPACING;
    }
    
    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }
}

impl fmt::Write for FramebufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl FramebufferWriter {
    /// Returns a raw pointer to the underlying framebuffer.
    /// Used by the GUI system to obtain a shared reference to the display memory.
    pub fn buffer_ptr(&self) -> *mut u8 {
        self.framebuffer.as_ptr() as *mut u8
    }

    pub fn buffer_len(&self) -> usize {
        self.framebuffer.len()
    }

    pub fn fb_info(&self) -> FrameBufferInfo {
        self.info
    }
}

pub fn init(framebuffer: &'static mut [u8], info: FrameBufferInfo) {
    let mut writer = FramebufferWriter::new(framebuffer, info);
    writer.clear();
    *FRAMEBUFFER_WRITER.lock() = Some(writer);
}
