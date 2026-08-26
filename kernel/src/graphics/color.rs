// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman. 
// All rights reserved.
// ==========================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color { 
    pub r: u8, 
    pub g: u8, 
    pub b: u8,
    pub a: u8, // Alpha channel (0 = fully transparent, 255 = fully opaque)
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b, a: 255 } }
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self { Self { r, g, b, a } }
    
    // Catalyst OS color palette (Future Sci-Fi Theme)
    pub const BLACK:       Color = Color::new(10, 10, 15);
    pub const WHITE:       Color = Color::new(240, 240, 245);
    pub const CATALYST_BLUE: Color = Color::new(30, 120, 255);  // Brand color
    pub const CATALYST_DARK: Color = Color::new(15, 20, 35);    // Desktop bg
    
    // Glassmorphism colors
    pub const GLASS_BG:    Color = Color::rgba(20, 20, 40, 150); // Semi-transparent dark
    pub const GLASS_BORDER: Color = Color::rgba(255, 255, 255, 30);
    
    pub const WINDOW_BG:   Color = Color::new(25, 30, 45);
    pub const WINDOW_TITLE: Color = Color::new(20, 25, 40);
    pub const ACCENT:      Color = Color::new(0, 200, 150);     // Teal accent
    pub const TEXT:        Color = Color::new(210, 215, 230);
    pub const GRAY:        Color = Color::new(80, 85, 100);
    pub const RED:         Color = Color::new(255, 80, 80);
    pub const GREEN:       Color = Color::new(80, 200, 80);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);

    // Alpha blend source over destination
    pub fn blend(src: Color, dst: Color) -> Color {
        if src.a == 255 { return src; }
        if src.a == 0 { return dst; }
        
        let alpha = src.a as u32;
        let inv_alpha = 255 - alpha;
        
        Color {
            r: ((src.r as u32 * alpha + dst.r as u32 * inv_alpha) / 255) as u8,
            g: ((src.g as u32 * alpha + dst.g as u32 * inv_alpha) / 255) as u8,
            b: ((src.b as u32 * alpha + dst.b as u32 * inv_alpha) / 255) as u8,
            a: 255, // Resulting color drawn to framebuffer is opaque
        }
    }
}
