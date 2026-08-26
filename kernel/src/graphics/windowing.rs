// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use super::geometry::{Rect, Region};
use super::color::Color;
use crate::events::InputEvent;
use crate::kprintln;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WindowId(pub u32);

pub struct Window {
    pub id: WindowId,
    pub parent_id: Option<WindowId>,
    pub first_child: Option<WindowId>,
    pub last_child: Option<WindowId>,
    pub next_sibling: Option<WindowId>,
    pub prev_sibling: Option<WindowId>,
    
    pub bounds: Rect,
    pub color: Color, // Temporary visual representation
}

impl Window {
    pub fn new(id: WindowId, bounds: Rect, color: Color) -> Self {
        Self {
            id,
            parent_id: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
            bounds,
            color,
        }
    }
}

pub const MAX_WINDOWS: usize = 256;

pub struct WindowManager {
    pub windows: [Option<Window>; MAX_WINDOWS],
    next_id: u32,
    
    pub root_id: Option<WindowId>,
    pub focused_window: Option<WindowId>,
    pub captured_window: Option<WindowId>,
    
    pub dirty_region: Region,
    
    // Metrics (P2C Definition of Done)
    pub metric_windows_created: usize,
    pub metric_windows_destroyed: usize,
    pub metric_full_redraws: usize,
    pub metric_cursor_invalidations: usize,
    pub metric_window_moves: usize,
    pub metric_hit_tests: usize,
    pub metric_pointer_captures: usize,
    
    // Cursor state
    pub cursor_rect: Rect,
}

impl WindowManager {
    pub const fn new() -> Self {
        const INIT: Option<Window> = None;
        Self {
            windows: [INIT; MAX_WINDOWS],
            next_id: 1, // 0 is reserved/invalid
            root_id: None,
            focused_window: None,
            captured_window: None,
            dirty_region: Region::new(),
            
            metric_windows_created: 0,
            metric_windows_destroyed: 0,
            metric_full_redraws: 0,
            metric_cursor_invalidations: 0,
            metric_window_moves: 0,
            metric_hit_tests: 0,
            metric_pointer_captures: 0,
            
            cursor_rect: Rect::new(0, 0, 10, 10),
        }
    }

    pub fn invalidate(&mut self, rect: Rect) {
        if !rect.is_empty() {
            self.dirty_region.add(rect);
        }
    }

    pub fn invalidate_all(&mut self, screen_width: i32, screen_height: i32) {
        self.invalidate(Rect::new(0, 0, screen_width, screen_height));
        self.metric_full_redraws += 1;
    }
    
    pub fn create_window(&mut self, bounds: Rect, color: Color, parent: Option<WindowId>) -> Option<WindowId> {
        let id_val = self.next_id;
        if (id_val as usize) >= MAX_WINDOWS {
            return None; // Out of memory in this bounded array
        }
        self.next_id += 1;
        let wid = WindowId(id_val);
        
        let mut new_win = Window::new(wid, bounds, color);
        new_win.parent_id = parent;
        
        // P2C-WM-01: Parent/Child linkage
        if let Some(pid) = parent {
            let last_child_id = self.windows[pid.0 as usize].as_ref().unwrap().last_child;
            
            if let Some(lc_id) = last_child_id {
                // Update old last child's next_sibling
                if let Some(old_last) = &mut self.windows[lc_id.0 as usize] {
                    old_last.next_sibling = Some(wid);
                    new_win.prev_sibling = Some(lc_id);
                }
            }
            
            // Now update parent
            if let Some(parent_win) = &mut self.windows[pid.0 as usize] {
                if parent_win.first_child.is_none() {
                    parent_win.first_child = Some(wid);
                }
                parent_win.last_child = Some(wid);
            }
        }
        
        self.windows[id_val as usize] = Some(new_win);
        self.metric_windows_created += 1;
        
        // Invalidate new window area
        self.invalidate(bounds);
        
        Some(wid)
    }
    
    pub fn destroy_window(&mut self, wid: WindowId) {
        // C1: Recursive subtree destruction
        self.unlink_window(wid);
        self.free_subtree(wid);
    }
    
    fn unlink_window(&mut self, wid: WindowId) {
        let (parent_id, prev_sibling, next_sibling) = match &self.windows[wid.0 as usize] {
            Some(w) => (w.parent_id, w.prev_sibling, w.next_sibling),
            None => return,
        };

        if let Some(pid) = parent_id {
            if let Some(parent_win) = &mut self.windows[pid.0 as usize] {
                if parent_win.first_child == Some(wid) {
                    parent_win.first_child = next_sibling;
                }
                if parent_win.last_child == Some(wid) {
                    parent_win.last_child = prev_sibling;
                }
            }
        }
        if let Some(prev) = prev_sibling {
            if let Some(prev_win) = &mut self.windows[prev.0 as usize] {
                prev_win.next_sibling = next_sibling;
            }
        }
        if let Some(next) = next_sibling {
            if let Some(next_win) = &mut self.windows[next.0 as usize] {
                next_win.prev_sibling = prev_sibling;
            }
        }
    }
    
    fn free_subtree(&mut self, wid: WindowId) {
        if let Some(win) = self.windows[wid.0 as usize].take() {
            let mut current_child = win.first_child;
            while let Some(child_id) = current_child {
                let next = if let Some(child_win) = &self.windows[child_id.0 as usize] {
                    child_win.next_sibling
                } else {
                    None
                };
                self.free_subtree(child_id);
                current_child = next;
            }
            
            // Clean up focus/capture state if they point to the destroyed window
            if self.focused_window == Some(wid) {
                self.focused_window = None;
            }
            if self.captured_window == Some(wid) {
                self.captured_window = None;
            }

            self.invalidate(win.bounds);
            self.metric_windows_destroyed += 1;
        }
    }

    pub fn move_window(&mut self, wid: WindowId, dx: i32, dy: i32) {
        if let Some(win) = &mut self.windows[wid.0 as usize] {
            let old_bounds = win.bounds;
            win.bounds.x += dx;
            win.bounds.y += dy;
            let new_bounds = win.bounds;
            
            // Invalidate old and new
            self.invalidate(old_bounds);
            self.invalidate(new_bounds);
            
            self.metric_window_moves += 1;
        }
    }

    pub fn update_cursor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        // P2C-CURSOR-01: Old + New Cursor Invalidation
        let old_cursor = self.cursor_rect;
        self.cursor_rect = Rect::new(x, y, width, height);
        
        self.invalidate(old_cursor);
        self.invalidate(self.cursor_rect);
        
        self.metric_cursor_invalidations += 1;
    }

    pub fn hit_test(&mut self, x: i32, y: i32) -> Option<WindowId> {
        self.metric_hit_tests += 1;
        
        // Start from root, recursive descent top-to-bottom
        // Since our Z-order is sibling based (first_child = bottom, last_child = top)
        // we must traverse children in reverse order (last_child -> prev_sibling).
        
        if let Some(root_id) = self.root_id {
            return self.hit_test_recursive(root_id, x, y);
        }
        None
    }
    
    fn hit_test_recursive(&self, node_id: WindowId, x: i32, y: i32) -> Option<WindowId> {
        let win = self.windows[node_id.0 as usize].as_ref()?;
        
        // If it doesn't intersect this window, it can't intersect children
        // (Assuming children are clipped by parents, standard constraint)
        if !win.bounds.contains(x, y) {
            return None;
        }
        
        // Check children top to bottom (last to first)
        let mut curr_child = win.last_child;
        while let Some(child_id) = curr_child {
            if let Some(hit) = self.hit_test_recursive(child_id, x, y) {
                return Some(hit);
            }
            if let Some(child_win) = self.windows[child_id.0 as usize].as_ref() {
                curr_child = child_win.prev_sibling;
            } else {
                curr_child = None;
            }
        }
        
        // If no child hit, but we are inside this window, this is the hit
        Some(node_id)
    }

    pub fn handle_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::MouseMove { dx, dy } => {
                // Determine pointer target
                let target = self.captured_window.or_else(|| self.hit_test(self.cursor_rect.x, self.cursor_rect.y));
                
                // If dragging a captured window (simple implementation for Phase 2C tests)
                if let Some(wid) = target {
                    if self.captured_window.is_some() {
                        self.move_window(wid, dx, dy);
                    }
                }
            },
            InputEvent::MouseButtonDown { button: _ } => {
                let target = self.hit_test(self.cursor_rect.x, self.cursor_rect.y);
                if let Some(wid) = target {
                    self.captured_window = Some(wid);
                    self.focused_window = Some(wid);
                    self.metric_pointer_captures += 1;
                }
            },
            InputEvent::MouseButtonUp { button: _ } => {
                self.captured_window = None;
            },
            _ => {}
        }
    }
    
    // Draw all windows within dirty region
    pub fn render(&mut self, canvas: &mut super::canvas::Canvas) {
        if self.dirty_region.is_empty() {
            return;
        }
        
        // Render root (bottom-up)
        if let Some(root_id) = self.root_id {
            for i in 0..self.dirty_region.count {
                let clip_rect = self.dirty_region.rects[i];
                self.render_recursive(root_id, canvas, &clip_rect);
            }
        }
        
        // Clear dirty region after render
        self.dirty_region.clear();
    }
    
    fn render_recursive(&self, node_id: WindowId, canvas: &mut super::canvas::Canvas, clip: &Rect) {
        let win = match self.windows[node_id.0 as usize].as_ref() {
            Some(w) => w,
            None => return,
        };
        
        // If window doesn't intersect clip, skip
        if !win.bounds.intersects(clip) {
            return;
        }
        
        // Draw this window's background (intersection of window bounds and clip rect)
        if let Some(draw_rect) = win.bounds.intersection(clip) {
            canvas.fill_rect(
                draw_rect.x as u32,
                draw_rect.y as u32,
                draw_rect.width as u32,
                draw_rect.height as u32,
                win.color
            );
        }
        
        // Draw children (bottom to top -> first_child to next_sibling)
        let mut curr_child = win.first_child;
        while let Some(child_id) = curr_child {
            self.render_recursive(child_id, canvas, clip);
            if let Some(child_win) = self.windows[child_id.0 as usize].as_ref() {
                curr_child = child_win.next_sibling;
            } else {
                curr_child = None;
            }
        }
    }
}
