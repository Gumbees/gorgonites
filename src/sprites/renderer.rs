//! Sprite rendering based on AI-generated descriptions

use macroquad::prelude::*;
use super::{SpriteDescription, SpriteShape, SpritePattern};

/// A rendered sprite with position
pub struct RenderedSprite {
    pub description: SpriteDescription,
    pub x: f32,
    pub y: f32,
    pub pixels: Vec<Vec<Option<[u8; 3]>>>,
}

impl RenderedSprite {
    /// Create a new rendered sprite from a description
    pub fn new(description: SpriteDescription, x: f32, y: f32) -> Self {
        let pixels = generate_pixels(&description);
        Self {
            description,
            x,
            y,
            pixels,
        }
    }

    /// Render this sprite
    pub fn render(&self) {
        let (w, h) = self.description.size;
        let scale = 2.0; // Scale up for visibility

        for (py, row) in self.pixels.iter().enumerate() {
            for (px, pixel) in row.iter().enumerate() {
                if let Some([r, g, b]) = pixel {
                    let color = Color::from_rgba(*r, *g, *b, 255);
                    draw_rectangle(
                        self.x + px as f32 * scale,
                        self.y + py as f32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }

        // Draw name below sprite
        let name_size = 10.0;
        draw_text(
            &self.description.name,
            self.x,
            self.y + h as f32 * scale + name_size + 2.0,
            name_size,
            WHITE,
        );
    }
}

/// Generate pixel data for a sprite
fn generate_pixels(desc: &SpriteDescription) -> Vec<Vec<Option<[u8; 3]>>> {
    let (w, h) = desc.size;
    let mut pixels = vec![vec![None; w as usize]; h as usize];

    match desc.shape {
        SpriteShape::Blob => generate_blob(&mut pixels, desc),
        SpriteShape::Humanoid => generate_humanoid(&mut pixels, desc),
        SpriteShape::Quadruped => generate_quadruped(&mut pixels, desc),
        SpriteShape::Flying => generate_flying(&mut pixels, desc),
        SpriteShape::Serpent => generate_serpent(&mut pixels, desc),
        SpriteShape::Geometric => generate_geometric(&mut pixels, desc),
    }

    // Apply pattern
    apply_pattern(&mut pixels, desc);

    pixels
}

/// Generate a blob-shaped sprite
fn generate_blob(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let rx = w as f32 / 2.0 - 1.0;
    let ry = h as f32 / 2.0 - 1.0;

    for y in 0..h as usize {
        for x in 0..w as usize {
            let dx = (x as f32 - cx) / rx;
            let dy = (y as f32 - cy) / ry;
            if dx * dx + dy * dy <= 1.0 {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Add eyes
    if w >= 12 && h >= 12 {
        let eye_y = (h / 3) as usize;
        let eye_x1 = (w / 3) as usize;
        let eye_x2 = (2 * w / 3) as usize;
        if eye_y < h as usize && eye_x1 < w as usize && eye_x2 < w as usize {
            pixels[eye_y][eye_x1] = Some(desc.accent_color);
            pixels[eye_y][eye_x2] = Some(desc.accent_color);
        }
    }
}

/// Generate a humanoid-shaped sprite
fn generate_humanoid(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let cx = w as usize / 2;

    // Head (top 1/4)
    let head_bottom = (h as usize / 4).max(1);
    let head_width = w as usize / 3;
    for y in 0..head_bottom {
        let start_x = cx.saturating_sub(head_width / 2);
        let end_x = (cx + head_width / 2 + 1).min(w as usize);
        for x in start_x..end_x {
            pixels[y][x] = Some(desc.primary_color);
        }
    }

    // Body (middle 1/2)
    let body_top = head_bottom;
    let body_bottom = 3 * h as usize / 4;
    let body_width = w as usize / 2;
    for y in body_top..body_bottom {
        let start_x = cx.saturating_sub(body_width / 2);
        let end_x = (cx + body_width / 2 + 1).min(w as usize);
        for x in start_x..end_x {
            pixels[y][x] = Some(desc.secondary_color);
        }
    }

    // Legs (bottom 1/4)
    let leg_width = (w as usize / 6).max(1);
    let leg_gap = w as usize / 6;
    for y in body_bottom..h as usize {
        // Left leg
        let left_start = cx.saturating_sub(leg_gap + leg_width);
        let left_end = cx.saturating_sub(leg_gap);
        for x in left_start..left_end {
            if x < w as usize {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
        // Right leg
        let right_start = cx + leg_gap;
        let right_end = (cx + leg_gap + leg_width).min(w as usize);
        for x in right_start..right_end {
            pixels[y][x] = Some(desc.primary_color);
        }
    }

    // Eyes
    if head_bottom > 2 && w >= 8 && cx > 1 {
        let eye_y = head_bottom / 2;
        if eye_y < h as usize && cx + 1 < w as usize {
            pixels[eye_y][cx - 1] = Some(desc.accent_color);
            pixels[eye_y][cx + 1] = Some(desc.accent_color);
        }
    }
}

/// Generate a quadruped sprite
fn generate_quadruped(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;

    // Body (horizontal oval)
    let body_top = h as usize / 3;
    let body_bottom = 2 * h as usize / 3;
    let body_left = w as usize / 6;
    let body_right = 5 * w as usize / 6;

    for y in body_top..body_bottom {
        for x in body_left..body_right {
            pixels[y][x] = Some(desc.primary_color);
        }
    }

    // Head (left side)
    let head_size = h as usize / 4;
    for y in (body_top - head_size / 2)..(body_top + head_size) {
        for x in 0..(body_left + head_size) {
            if y < h as usize && x < w as usize {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Legs
    let leg_width = w as usize / 8;
    for y in body_bottom..h as usize {
        // Front legs
        for x in (body_left)..(body_left + leg_width) {
            if x < w as usize {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
        // Back legs
        for x in (body_right - leg_width)..body_right {
            if x < w as usize {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Eye
    let eye_y = body_top;
    let eye_x = body_left / 2;
    if eye_y < h as usize && eye_x < w as usize {
        pixels[eye_y][eye_x] = Some(desc.accent_color);
    }
}

/// Generate a flying sprite (with wings)
fn generate_flying(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let cx = w as usize / 2;
    let cy = h as usize / 2;

    // Body (small oval in center)
    let body_w = w as usize / 4;
    let body_h = h as usize / 3;
    for y in (cy - body_h / 2)..(cy + body_h / 2) {
        for x in (cx - body_w / 2)..(cx + body_w / 2) {
            if y < h as usize && x < w as usize {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Wings (triangular)
    // Left wing
    let wing_y_start = cy.saturating_sub(body_h);
    let wing_y_end = cy + body_h / 2;
    for y in wing_y_start..wing_y_end {
        let wing_extent = ((cy + body_h / 2) as i32 - y as i32).abs() as usize;
        let wing_min_x = (cx.saturating_sub(body_w / 2)).saturating_sub(wing_extent + 2);
        for x in 0..cx.saturating_sub(body_w / 2) {
            if y < h as usize && x < w as usize && x > wing_min_x {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Right wing
    for y in wing_y_start..wing_y_end {
        let wing_extent = ((cy + body_h / 2) as i32 - y as i32).abs() as usize;
        let wing_max_x = (cx + body_w / 2 + wing_extent + 2).min(w as usize);
        for x in (cx + body_w / 2)..w as usize {
            if y < h as usize && x < wing_max_x {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Eyes
    if cy > 0 && cx > 0 {
        let eye_y = cy - 1;
        if eye_y < h as usize {
            if cx > 0 && cx - 1 < w as usize {
                pixels[eye_y][cx - 1] = Some(desc.accent_color);
            }
            if cx + 1 < w as usize {
                pixels[eye_y][cx + 1] = Some(desc.accent_color);
            }
        }
    }
}

/// Generate a serpent sprite
fn generate_serpent(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let cy = h as usize / 2;
    let thickness = (h as usize / 4).max(2);

    // Wavy body
    for x in 0..w as usize {
        let wave = ((x as f32 * 0.5).sin() * (h as f32 / 4.0)) as i32;
        let y_center = (cy as i32 + wave) as usize;

        for dy in 0..thickness {
            let y = y_center + dy;
            if y < h as usize {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Head (larger at start)
    let head_size = thickness + 2;
    for y in (cy.saturating_sub(head_size / 2))..(cy + head_size / 2) {
        for x in 0..head_size {
            if y < h as usize && x < w as usize {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Eye
    if cy < h as usize && 2 < w as usize {
        pixels[cy][2] = Some(desc.accent_color);
    }
}

/// Generate a geometric sprite
fn generate_geometric(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let cx = w as usize / 2;
    let cy = h as usize / 2;

    // Diamond shape
    let size = w.min(h) as usize / 2;

    for y in 0..h as usize {
        for x in 0..w as usize {
            let dx = (x as i32 - cx as i32).abs();
            let dy = (y as i32 - cy as i32).abs();

            if dx + dy <= size as i32 {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Inner diamond with secondary color
    let inner_size = size / 2;
    for y in 0..h as usize {
        for x in 0..w as usize {
            let dx = (x as i32 - cx as i32).abs();
            let dy = (y as i32 - cy as i32).abs();

            if dx + dy <= inner_size as i32 {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Center dot
    if cy < h as usize && cx < w as usize {
        pixels[cy][cx] = Some(desc.accent_color);
    }
}

/// Apply pattern overlay to pixels
fn apply_pattern(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;

    match desc.pattern {
        SpritePattern::Solid => {
            // Already solid, no changes
        }
        SpritePattern::Striped => {
            for y in 0..h as usize {
                if y % 3 == 0 {
                    for x in 0..w as usize {
                        if pixels[y][x].is_some() {
                            pixels[y][x] = Some(desc.secondary_color);
                        }
                    }
                }
            }
        }
        SpritePattern::Spotted => {
            for y in (0..h as usize).step_by(3) {
                for x in (0..w as usize).step_by(3) {
                    if pixels[y][x].is_some() {
                        pixels[y][x] = Some(desc.accent_color);
                    }
                }
            }
        }
        SpritePattern::Gradient => {
            for y in 0..h as usize {
                let factor = y as f32 / h as f32;
                for x in 0..w as usize {
                    if let Some(color) = pixels[y][x] {
                        let blended = blend_colors(color, desc.secondary_color, factor);
                        pixels[y][x] = Some(blended);
                    }
                }
            }
        }
        SpritePattern::Checkered => {
            for y in 0..h as usize {
                for x in 0..w as usize {
                    if (x + y) % 2 == 0 && pixels[y][x].is_some() {
                        pixels[y][x] = Some(desc.secondary_color);
                    }
                }
            }
        }
    }
}

/// Blend two colors
fn blend_colors(c1: [u8; 3], c2: [u8; 3], factor: f32) -> [u8; 3] {
    [
        ((c1[0] as f32 * (1.0 - factor) + c2[0] as f32 * factor) as u8),
        ((c1[1] as f32 * (1.0 - factor) + c2[1] as f32 * factor) as u8),
        ((c1[2] as f32 * (1.0 - factor) + c2[2] as f32 * factor) as u8),
    ]
}

/// Manages all rendered sprites on screen
pub struct SpriteManager {
    sprites: Vec<RenderedSprite>,
    max_sprites: usize,
}

impl SpriteManager {
    pub fn new(max_sprites: usize) -> Self {
        Self {
            sprites: Vec::new(),
            max_sprites,
        }
    }

    /// Add a new sprite
    pub fn add(&mut self, sprite: RenderedSprite) {
        if self.sprites.len() >= self.max_sprites {
            self.sprites.remove(0); // Remove oldest
        }
        self.sprites.push(sprite);
    }

    /// Render all sprites
    pub fn render(&self) {
        for sprite in &self.sprites {
            sprite.render();
        }
    }

    /// Get sprite count
    pub fn count(&self) -> usize {
        self.sprites.len()
    }

    /// Clear all sprites
    pub fn clear(&mut self) {
        self.sprites.clear();
    }
}
