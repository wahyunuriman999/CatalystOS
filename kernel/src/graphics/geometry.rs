// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

use core::cmp::{max, min};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    pub fn area(&self) -> i32 {
        if self.is_empty() {
            0
        } else {
            self.width * self.height
        }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        !(self.x + self.width <= other.x || 
          other.x + other.width <= self.x ||
          self.y + self.height <= other.y || 
          other.y + other.height <= self.y)
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }
        let x = max(self.x, other.x);
        let y = max(self.y, other.y);
        let right = min(self.x + self.width, other.x + other.width);
        let bottom = min(self.y + self.height, other.y + other.height);
        
        Some(Rect::new(x, y, right - x, bottom - y))
    }

    pub fn union(&self, other: &Rect) -> Rect {
        if self.is_empty() { return *other; }
        if other.is_empty() { return *self; }

        let x = min(self.x, other.x);
        let y = min(self.y, other.y);
        let right = max(self.x + self.width, other.x + other.width);
        let bottom = max(self.y + self.height, other.y + other.height);

        Rect::new(x, y, right - x, bottom - y)
    }
}

pub const MAX_DIRTY_RECTS: usize = 16;

pub struct Region {
    pub rects: [Rect; MAX_DIRTY_RECTS],
    pub count: usize,
    pub merges_performed: usize,
}

impl Region {
    pub const fn new() -> Self {
        Self {
            rects: [Rect::new(0,0,0,0); MAX_DIRTY_RECTS],
            count: 0,
            merges_performed: 0,
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.merges_performed = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Add a rect to the dirty region. Implements bounded optimization budget heuristic.
    pub fn add(&mut self, new_rect: Rect) {
        if new_rect.is_empty() { return; }

        // 1. Try to find a highly overlapping or adjacent rect to merge with.
        let mut best_merge_idx = None;
        let mut best_cost = i32::MAX;

        for i in 0..self.count {
            let r = &self.rects[i];
            let union_rect = r.union(&new_rect);
            
            let combined_area = union_rect.area();
            let sum_areas = r.area() + new_rect.area();
            
            // Heuristic: If bounding box area is not significantly larger than the sum,
            // or if it's already bounded budget, merge them.
            // Cost is how much "wasted" space we draw if we merge.
            let waste = combined_area - sum_areas;
            
            // If they intersect, merging is usually good.
            if waste < best_cost {
                best_cost = waste;
                best_merge_idx = Some(i);
            }
        }

        // 2. Decide whether to merge or append.
        if let Some(idx) = best_merge_idx {
            let r = &self.rects[idx];
            // If we have room, only merge if waste is low. 
            // If waste is high (distant rects), prefer appending.
            if self.count < MAX_DIRTY_RECTS && best_cost > 4096 { // e.g. 64x64 pixels of waste threshold
                self.rects[self.count] = new_rect;
                self.count += 1;
            } else {
                // Forced to merge (budget hit) or waste is low
                self.rects[idx] = self.rects[idx].union(&new_rect);
                self.merges_performed += 1;
                self.collapse_if_needed();
            }
        } else {
            // First rect
            self.rects[0] = new_rect;
            self.count = 1;
        }
    }

    /// P2C-PERF-02: Bounded Optimization Budget
    /// If we have too many rects, we collapse them down.
    fn collapse_if_needed(&mut self) {
        if self.count <= MAX_DIRTY_RECTS / 2 { return; }
        
        // Very basic bounded collapse: if we are filling up the array, 
        // aggressively merge the two closest rects.
        // For simplicity in Phase 2C, if we hit max, we just union all into a single bounding box.
        if self.count >= MAX_DIRTY_RECTS {
            let mut bounding_box = self.rects[0];
            for i in 1..self.count {
                bounding_box = bounding_box.union(&self.rects[i]);
            }
            self.rects[0] = bounding_box;
            self.count = 1;
            self.merges_performed += 1;
        }
    }
}
