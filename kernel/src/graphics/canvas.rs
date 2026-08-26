// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

use crate::graphics::color::Color;

// Global framebuffer storage for GUI rendering
static mut FB_PTR: *mut u8 = core::ptr::null_mut();
static mut FB_LEN: usize = 0;
static FB_WIDTH:  core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
static FB_HEIGHT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
static FB_BPP:    core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(3);
static FB_STRIDE: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
static FB_FORMAT: spin::Mutex<bootloader_api::info::PixelFormat> =
    spin::Mutex::new(bootloader_api::info::PixelFormat::Bgr);

pub fn store_framebuffer(buf: &'static mut [u8], info: &bootloader_api::info::FrameBufferInfo) {
    unsafe {
        FB_PTR = buf.as_mut_ptr();
        FB_LEN = buf.len();
    }
    use core::sync::atomic::Ordering::Relaxed;
    FB_WIDTH.store(info.width as u32, Relaxed);
    FB_HEIGHT.store(info.height as u32, Relaxed);
    FB_BPP.store(info.bytes_per_pixel as u32, Relaxed);
    FB_STRIDE.store(info.stride as u32, Relaxed);
    *FB_FORMAT.lock() = info.pixel_format;
}

pub fn with_canvas<F: FnOnce(&mut Canvas<'_>)>(f: F) {
    use core::sync::atomic::Ordering::Relaxed;
    let ptr = unsafe { FB_PTR };
    let len = unsafe { FB_LEN };
    if ptr.is_null() || len == 0 { return; }
    let buf = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
    let mut canvas = Canvas {
        buffer: buf,
        width:  FB_WIDTH.load(Relaxed),
        height: FB_HEIGHT.load(Relaxed),
        bytes_per_pixel: FB_BPP.load(Relaxed),
        stride: FB_STRIDE.load(Relaxed),
        pixel_format: *FB_FORMAT.lock(),
    };
    f(&mut canvas);
}

pub struct Canvas<'a> {
    pub buffer: &'a mut [u8],
    pub width:  u32,
    pub height: u32,
    pub bytes_per_pixel: u32,
    pub stride: u32,
    pub pixel_format: bootloader_api::info::PixelFormat,
}

impl<'a> Canvas<'a> {
    #[inline(always)]
    pub fn write_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height { return; }
        let offset = ((y * self.stride + x) * self.bytes_per_pixel) as usize;
        if offset + 2 >= self.buffer.len() { return; }
        
        let existing = match self.pixel_format {
            bootloader_api::info::PixelFormat::Bgr => Color::new(self.buffer[offset+2], self.buffer[offset+1], self.buffer[offset]),
            _ => Color::new(self.buffer[offset], self.buffer[offset+1], self.buffer[offset+2]),
        };
        
        let blended = Color::blend(color, existing);

        match self.pixel_format {
            bootloader_api::info::PixelFormat::Bgr => {
                self.buffer[offset]     = blended.b;
                self.buffer[offset + 1] = blended.g;
                self.buffer[offset + 2] = blended.r;
            }
            _ => {
                self.buffer[offset]     = blended.r;
                self.buffer[offset + 1] = blended.g;
                self.buffer[offset + 2] = blended.b;
            }
        }
    }

    pub fn fill(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.write_pixel(x, y, color);
            }
        }
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for dy in 0..h {
            for dx in 0..w {
                self.write_pixel(x + dx, y + dy, color);
            }
        }
    }

    pub fn draw_rect_outline(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for dx in 0..w { self.write_pixel(x + dx, y, color); self.write_pixel(x + dx, y + h.saturating_sub(1), color); }
        for dy in 0..h { self.write_pixel(x, y + dy, color); self.write_pixel(x + w.saturating_sub(1), y + dy, color); }
    }

    pub fn fill_rounded_rect(&mut self, x: u32, y: u32, w: u32, h: u32, r: u32, color: Color) {
        let r = r as i32;
        let r_sq = r * r;
        
        for dy in 0..h {
            for dx in 0..w {
                let mut draw = true;
                let c_x = dx as i32;
                let c_y = dy as i32;
                
                // Top-left
                if c_x < r && c_y < r {
                    if (r - c_x)*(r - c_x) + (r - c_y)*(r - c_y) > r_sq { draw = false; }
                }
                // Top-right
                else if c_x >= (w as i32 - r) && c_y < r {
                    let rx = c_x - (w as i32 - r - 1);
                    if rx*rx + (r - c_y)*(r - c_y) > r_sq { draw = false; }
                }
                // Bottom-left
                else if c_x < r && c_y >= (h as i32 - r) {
                    let ry = c_y - (h as i32 - r - 1);
                    if (r - c_x)*(r - c_x) + ry*ry > r_sq { draw = false; }
                }
                // Bottom-right
                else if c_x >= (w as i32 - r) && c_y >= (h as i32 - r) {
                    let rx = c_x - (w as i32 - r - 1);
                    let ry = c_y - (h as i32 - r - 1);
                    if rx*rx + ry*ry > r_sq { draw = false; }
                }
                
                if draw {
                    self.write_pixel(x + dx, y + dy, color);
                }
            }
        }
    }

    pub fn draw_horizontal_line(&mut self, x: u32, y: u32, len: u32, color: Color) {
        for i in 0..len { self.write_pixel(x + i, y, color); }
    }

    pub fn draw_vertical_line(&mut self, x: u32, y: u32, len: u32, color: Color) {
        for i in 0..len { self.write_pixel(x, y + i, color); }
    }

    pub fn draw_circle(&mut self, cx: u32, cy: u32, r: u32, color: Color) {
        let r = r as i32;
        let cx = cx as i32; let cy = cy as i32;
        for y in -r..=r {
            for x in -r..=r {
                if x*x + y*y <= r*r {
                    self.write_pixel((cx + x) as u32, (cy + y) as u32, color);
                }
            }
        }
    }

    pub fn draw_char(&mut self, x: u32, y: u32, ch: char, color: Color) {
        let c = ch as usize;
        if c < 32 || c > 127 { return; }
        let idx = c - 32;
        if idx >= FONT_DATA.len() / 16 { return; }
        for row in 0..16u32 {
            let byte = FONT_DATA[idx * 16 + row as usize];
            for col in 0..8u32 {
                if byte & (0x80 >> col) != 0 {
                    self.write_pixel(x + col, y + row, color);
                }
            }
        }
    }

    pub fn draw_str(&mut self, mut x: u32, y: u32, s: &str, color: Color) {
        for ch in s.chars() {
            if ch == '\n' { return; }
            self.draw_char(x, y, ch, color);
            x += 8;
            if x + 8 >= self.width { break; }
        }
    }

    pub fn draw_str_wrapped(&mut self, x: u32, y: u32, s: &str, color: Color, max_width: u32) {
        let mut cx = x; let mut cy = y;
        for word in s.split_whitespace() {
            let wlen = word.len() as u32 * 8;
            if cx + wlen > x + max_width { cx = x; cy += 18; }
            self.draw_str(cx, cy, word, color);
            cx += wlen + 8;
        }
    }

    pub fn draw_char_scaled(&mut self, x: u32, y: u32, ch: char, color: Color, scale: u32) {
        let c = ch as usize;
        if c < 32 || c > 127 { return; }
        let idx = c - 32;
        if idx >= FONT_DATA.len() / 16 { return; }
        for row in 0..16u32 {
            let byte = FONT_DATA[idx * 16 + row as usize];
            for col in 0..8u32 {
                if byte & (0x80 >> col) != 0 {
                    self.fill_rect(x + col*scale, y + row*scale, scale, scale, color);
                }
            }
        }
    }

    pub fn draw_str_scaled(&mut self, mut x: u32, y: u32, s: &str, color: Color, scale: u32) {
        for ch in s.chars() {
            if ch == '\n' { return; }
            self.draw_char_scaled(x, y, ch, color, scale);
            x += 8 * scale;
            if x + (8*scale) >= self.width { break; }
        }
    }
}

// Embedded 8x16 PC VGA font (ASCII 32-127, 96 chars × 16 bytes)
// This is the classic IBM VGA 8x16 font, public domain
static FONT_DATA: &[u8] = include_bytes!("font8x16.bin");
