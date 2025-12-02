//! Map rendering

use macroquad::prelude::*;
use super::palette;

/// Render a tile-based map
pub struct MapRenderer {
    /// Tile size in pixels
    pub tile_size: f32,

    /// Show grid lines
    pub show_grid: bool,

    /// Grid line color
    pub grid_color: Color,
}

impl Default for MapRenderer {
    fn default() -> Self {
        Self {
            tile_size: 32.0,
            show_grid: true,
            grid_color: palette::GRID,
        }
    }
}

impl MapRenderer {
    pub fn new(tile_size: f32) -> Self {
        Self {
            tile_size,
            ..Default::default()
        }
    }

    /// Render grid overlay
    pub fn render_grid(&self, viewport: Rect) {
        if !self.show_grid {
            return;
        }

        let start_x = (viewport.x / self.tile_size).floor() * self.tile_size;
        let start_y = (viewport.y / self.tile_size).floor() * self.tile_size;
        let end_x = viewport.x + viewport.w;
        let end_y = viewport.y + viewport.h;

        let mut x = start_x;
        while x <= end_x {
            draw_line(x, viewport.y, x, end_y, 1.0, self.grid_color);
            x += self.tile_size;
        }

        let mut y = start_y;
        while y <= end_y {
            draw_line(viewport.x, y, end_x, y, 1.0, self.grid_color);
            y += self.tile_size;
        }
    }

    /// Render a single tile
    pub fn render_tile(&self, x: i32, y: i32, color: Color) {
        let px = x as f32 * self.tile_size;
        let py = y as f32 * self.tile_size;
        draw_rectangle(px, py, self.tile_size, self.tile_size, color);
    }

    /// Render a tile with border
    pub fn render_tile_bordered(&self, x: i32, y: i32, fill: Color, border: Color) {
        let px = x as f32 * self.tile_size;
        let py = y as f32 * self.tile_size;
        draw_rectangle(px, py, self.tile_size, self.tile_size, fill);
        draw_rectangle_lines(px, py, self.tile_size, self.tile_size, 1.0, border);
    }

    /// Convert world position to tile coordinates
    pub fn world_to_tile(&self, world_x: f32, world_y: f32) -> (i32, i32) {
        (
            (world_x / self.tile_size).floor() as i32,
            (world_y / self.tile_size).floor() as i32,
        )
    }

    /// Convert tile coordinates to world position (center of tile)
    pub fn tile_to_world(&self, tile_x: i32, tile_y: i32) -> (f32, f32) {
        (
            tile_x as f32 * self.tile_size + self.tile_size / 2.0,
            tile_y as f32 * self.tile_size + self.tile_size / 2.0,
        )
    }
}

/// Colors for different terrain types
pub fn terrain_color(terrain: &str) -> Color {
    match terrain {
        "plains" => Color::from_rgba(120, 180, 80, 255),
        "hills" => Color::from_rgba(140, 120, 80, 255),
        "mountains" => Color::from_rgba(100, 100, 110, 255),
        "forest" => Color::from_rgba(40, 120, 40, 255),
        "desert" => Color::from_rgba(220, 200, 140, 255),
        "tundra" => Color::from_rgba(200, 220, 230, 255),
        "swamp" => Color::from_rgba(80, 100, 60, 255),
        "water" => Color::from_rgba(60, 100, 180, 255),
        "deep_water" => Color::from_rgba(30, 60, 140, 255),
        _ => Color::from_rgba(128, 128, 128, 255),
    }
}
