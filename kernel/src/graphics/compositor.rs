// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use alloc::vec::Vec;
use crate::graphics::canvas::Canvas;
use crate::graphics::color::Color;
use bootloader_api::info::PixelFormat;
use spin::Mutex;
use lazy_static::lazy_static;

pub struct Compositor {
    pub backbuffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub bytes_per_pixel: u32,
    pub stride: u32,
    pub format: PixelFormat,
}

lazy_static! {
    pub static ref COMPOSITOR: Mutex<Option<Compositor>> = Mutex::new(None);
}

pub fn init_compositor(width: u32, height: u32, bpp: u32, stride: u32, format: PixelFormat) {
    let size = (height * stride * bpp) as usize;
    crate::kprintln!("[COMPOSITOR] Allocating {} bytes for backbuffer ({}x{})", size, width, height);
    
    // Allocate backbuffer filled with zeroes
    let backbuffer = alloc::vec![0; size];
    
    let comp = Compositor {
        backbuffer,
        width,
        height,
        bytes_per_pixel: bpp,
        stride,
        format,
    };
    
    *COMPOSITOR.lock() = Some(comp);
    crate::kprintln!("[COMPOSITOR] Initialized.");
}

impl Compositor {
    /// Gets a Canvas representing the backbuffer so we can draw onto it
    pub fn get_canvas(&mut self) -> Canvas<'_> {
        Canvas {
            buffer: &mut self.backbuffer,
            width: self.width,
            height: self.height,
            bytes_per_pixel: self.bytes_per_pixel,
            stride: self.stride,
            pixel_format: self.format,
        }
    }
    
    /// Swaps (copies) the backbuffer to the physical frontbuffer
    pub fn flush(&self, frontbuffer: &mut [u8]) {
        // Simple memory copy
        let len = core::cmp::min(self.backbuffer.len(), frontbuffer.len());
        unsafe {
            core::ptr::copy_nonoverlapping(self.backbuffer.as_ptr(), frontbuffer.as_mut_ptr(), len);
        }
    }
}
