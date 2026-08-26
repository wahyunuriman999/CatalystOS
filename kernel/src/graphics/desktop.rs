// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use spin::Mutex;
use crate::graphics::color::Color;
use crate::graphics::geometry::Rect;
use crate::graphics::windowing::{WindowManager, WindowId};
use crate::graphics::canvas::Canvas;
use crate::events::InputEvent;

pub static DESKTOP: Mutex<Desktop> = Mutex::new(Desktop::new());

// Temporary static values for Phase 2C to match screen size
const SCREEN_WIDTH: i32 = 1280;
const SCREEN_HEIGHT: i32 = 720;

pub struct Desktop {
    pub wm: WindowManager,
    cursor_x: i32,
    cursor_y: i32,
    initialized: bool,
    needs_full_redraw: bool,
}

impl Desktop {
    pub const fn new() -> Self {
        Self {
            wm: WindowManager::new(),
            cursor_x: SCREEN_WIDTH / 2,
            cursor_y: SCREEN_HEIGHT / 2,
            initialized: false,
            needs_full_redraw: true,
        }
    }

    pub fn init(&mut self) {
        if self.initialized { return; }
        self.initialized = true;

        // Root window (desktop background)
        let root_bounds = Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
        let root_color = Color::new(20, 20, 25);
        let root_id = self.wm.create_window(root_bounds, root_color, None).unwrap();
        self.wm.root_id = Some(root_id);

        // Window A
        let win_a = Rect::new(100, 100, 400, 300);
        let id_a = self.wm.create_window(win_a, Color::new(40, 40, 45), Some(root_id)).unwrap();
        
        // Window A Child (Button)
        let btn_a = Rect::new(120, 120, 100, 40);
        self.wm.create_window(btn_a, Color::new(80, 120, 200), Some(id_a)).unwrap();

        // Window B
        let win_b = Rect::new(300, 200, 400, 300);
        self.wm.create_window(win_b, Color::new(50, 50, 55), Some(root_id)).unwrap();

        self.needs_full_redraw = true;
    }

    pub fn handle_event(&mut self, event: InputEvent) {
        if !self.initialized {
            self.init();
        }

        // Pass to WindowManager to handle capture/focus/hit test
        match event {
            InputEvent::MouseMove { dx, dy } => {
                let old_x = self.cursor_x;
                let old_y = self.cursor_y;
                
                self.cursor_x = (self.cursor_x + dx).clamp(0, SCREEN_WIDTH - 1);
                self.cursor_y = (self.cursor_y - dy).clamp(0, SCREEN_HEIGHT - 1); // PS/2 y is inverted
                
                // Route input before updating cursor visual (so logic uses old coords or new, doesn't matter much)
                self.wm.handle_input(InputEvent::MouseMove { dx: self.cursor_x - old_x, dy: self.cursor_y - old_y });
                
                // P2C-CURSOR-01: Update cursor bounding box triggers double invalidation
                self.wm.update_cursor(self.cursor_x, self.cursor_y, 10, 10);
            },
            InputEvent::MouseButtonDown { button } => {
                self.wm.handle_input(event);
            },
            InputEvent::MouseButtonUp { button } => {
                self.wm.handle_input(event);
            },
            _ => {}
        }
    }

    pub fn draw_if_dirty(&mut self, canvas: &mut Canvas) -> bool {
        if !self.initialized {
            self.init();
        }
        
        if self.needs_full_redraw {
            self.wm.invalidate_all(SCREEN_WIDTH, SCREEN_HEIGHT);
            self.needs_full_redraw = false;
        }

        if self.wm.dirty_region.is_empty() {
            return false;
        }

        // Draw windows within dirty region
        self.wm.render(canvas);
        
        // Draw cursor on top of everything (Phase 2C basic rendering)
        // Note: Real cursor rendering might use hardware cursor or overlay
        canvas.fill_rect(
            self.cursor_x as u32,
            self.cursor_y as u32,
            10,
            10,
            Color::new(255, 255, 255)
        );
        
        true
    }
    
    pub fn get_metrics(&self) -> (usize, usize, usize, usize, usize, usize, usize) {
        (
            self.wm.metric_windows_created,
            self.wm.dirty_region.merges_performed,
            self.wm.metric_full_redraws,
            self.wm.metric_cursor_invalidations,
            self.wm.metric_window_moves,
            self.wm.metric_hit_tests,
            self.wm.metric_pointer_captures,
        )
    }
    
    pub fn is_dirty(&self) -> bool {
        !self.wm.dirty_region.is_empty() || self.needs_full_redraw
    }
}
