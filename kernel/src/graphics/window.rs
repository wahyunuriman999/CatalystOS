// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use alloc::string::String;
use alloc::vec::Vec;
use crate::graphics::canvas::Canvas;
use crate::graphics::color::Color;

#[derive(Clone, Copy, PartialEq)]
pub enum WindowState { Normal, Minimized, Maximized, Focused }

pub struct Window {
    pub id: u32,
    pub title: &'static str,
    pub x: u32, pub y: u32,
    pub width: u32, pub height: u32,
    pub state: WindowState,
    pub back_buffer: Vec<u8>,
    pub dirty: bool,
    pub text_content: String,
}

impl Window {
    pub fn new(id: u32, title: &'static str, x: u32, y: u32, width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize; // 4 bytes per pixel for safety
        Self {
            id, title, x, y, width, height,
            state: WindowState::Normal,
            back_buffer: alloc::vec![0; size],
            dirty: true,
            text_content: String::new(),
        }
    }

    pub fn draw_title_bar(&self, canvas: &mut Canvas) {
        // Height: 28px
        // Background: WINDOW_TITLE color
        canvas.fill_rect(self.x, self.y, self.width, 28, Color::WINDOW_TITLE);
        
        // Left side: colored circle accent (CATALYST_BLUE, 8px radius approx with a square) at x+14, y+14 (center)
        canvas.fill_rect(self.x + 8, self.y + 8, 12, 12, Color::CATALYST_BLUE);
        
        // Title text: centered, TEXT color
        // Roughly approx text width (8px per char)
        let text_w = (self.title.len() as u32) * 8;
        let text_x = if self.width > text_w { self.x + (self.width - text_w) / 2 } else { self.x + 30 };
        canvas.draw_str(text_x, self.y + 6, self.title, Color::TEXT);
        
        // Right side: 3 window control circles
        canvas.fill_rect(self.x + self.width - 24, self.y + 8, 12, 12, Color::RED); // Close
        canvas.fill_rect(self.x + self.width - 44, self.y + 8, 12, 12, Color::GRAY); // Minimize
        canvas.fill_rect(self.x + self.width - 64, self.y + 8, 12, 12, Color::GREEN); // Maximize
    }

    pub fn draw_chrome(&self, canvas: &mut Canvas) {
        // Content background
        canvas.fill_rect(self.x, self.y + 28, self.width, self.height - 28, Color::WINDOW_BG);
        // Border
        canvas.draw_rect_outline(self.x, self.y, self.width, self.height, Color::GRAY);
        self.draw_title_bar(canvas);
        
        // Draw content text
        if !self.text_content.is_empty() {
            canvas.draw_str(self.x + 10, self.y + 38, &self.text_content, Color::TEXT);
        }
    }

    pub fn write_content_text(&mut self, text: &str) {
        self.text_content.push_str(text);
        self.dirty = true;
    }
}
