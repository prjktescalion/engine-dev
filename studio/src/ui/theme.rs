//! Dark-theme color palette, mirroring the legacy editor.

use gpui::Rgba;

pub const BG: u32 = 0x0a0e14;
pub const PANEL: u32 = 0x11161d;
pub const PANEL_ALT: u32 = 0x161c24;
pub const BORDER: u32 = 0x1c2333;
pub const TEXT: u32 = 0xc9d1d9;
pub const TEXT_DIM: u32 = 0x7d8590;
pub const ACCENT: u32 = 0x58a6ff;
pub const ACCENT_DIM: u32 = 0x2a4365;
pub const GRID: u32 = 0x1c2333;
pub const DANGER: u32 = 0xf85149;

pub fn rgba(hex: u32) -> Rgba {
    let r = ((hex >> 16) & 0xff) as f32 / 255.0;
    let g = ((hex >> 8) & 0xff) as f32 / 255.0;
    let b = (hex & 0xff) as f32 / 255.0;
    Rgba { r, g, b, a: 1.0 }
}
