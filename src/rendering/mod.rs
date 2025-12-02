//! Rendering systems using Macroquad

mod camera;
mod sprites;
mod map;

pub use camera::*;
pub use sprites::*;
pub use map::*;

use macroquad::prelude::*;

/// Rendering configuration
pub struct RenderConfig {
    /// Show grid overlay
    pub show_grid: bool,

    /// Grid cell size in pixels
    pub grid_size: f32,

    /// UI scale factor
    pub ui_scale: f32,

    /// Enable visual effects
    pub effects_enabled: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            show_grid: true,
            grid_size: 32.0,
            ui_scale: 1.0,
            effects_enabled: true,
        }
    }
}

/// Color palette for the game
pub mod palette {
    use macroquad::prelude::Color;

    pub const BACKGROUND: Color = Color::new(0.08, 0.08, 0.12, 1.0);
    pub const GRID: Color = Color::new(0.16, 0.16, 0.20, 1.0);
    pub const UI_BACKGROUND: Color = Color::new(0.12, 0.12, 0.16, 0.9);
    pub const UI_BORDER: Color = Color::new(0.3, 0.3, 0.35, 1.0);
    pub const TEXT_PRIMARY: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const TEXT_SECONDARY: Color = Color::new(0.7, 0.7, 0.7, 1.0);
    pub const HIGHLIGHT: Color = Color::new(0.4, 0.6, 1.0, 1.0);
    pub const WARNING: Color = Color::new(1.0, 0.8, 0.2, 1.0);
    pub const DANGER: Color = Color::new(1.0, 0.3, 0.3, 1.0);
    pub const SUCCESS: Color = Color::new(0.3, 1.0, 0.4, 1.0);
}

/// Render a progress bar
pub fn draw_progress_bar(x: f32, y: f32, width: f32, height: f32, progress: f32, color: Color) {
    // Background
    draw_rectangle(x, y, width, height, palette::UI_BACKGROUND);

    // Fill
    let fill_width = width * progress.clamp(0.0, 1.0);
    draw_rectangle(x, y, fill_width, height, color);

    // Border
    draw_rectangle_lines(x, y, width, height, 1.0, palette::UI_BORDER);
}

/// Render text with a shadow for readability
pub fn draw_text_shadowed(text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text(text, x + 1.0, y + 1.0, size, BLACK);
    draw_text(text, x, y, size, color);
}
