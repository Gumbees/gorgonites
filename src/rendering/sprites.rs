//! Sprite and texture management

use macroquad::prelude::*;
use std::collections::HashMap;

/// Manages loaded textures
pub struct SpriteManager {
    textures: HashMap<String, Texture2D>,
    missing_texture: Option<Texture2D>,
}

impl SpriteManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            missing_texture: None,
        }
    }

    /// Initialize with a placeholder texture for missing sprites
    pub async fn init(&mut self) {
        // Create a simple checkerboard pattern for missing textures
        let size = 32;
        let mut pixels = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                let is_magenta = (x / 4 + y / 4) % 2 == 0;
                if is_magenta {
                    pixels[idx] = 255;     // R
                    pixels[idx + 1] = 0;   // G
                    pixels[idx + 2] = 255; // B
                } else {
                    pixels[idx] = 0;       // R
                    pixels[idx + 1] = 0;   // G
                    pixels[idx + 2] = 0;   // B
                }
                pixels[idx + 3] = 255; // A
            }
        }

        let texture = Texture2D::from_rgba8(size as u16, size as u16, &pixels);
        texture.set_filter(FilterMode::Nearest);
        self.missing_texture = Some(texture);
    }

    /// Load a texture from file
    pub async fn load(&mut self, id: &str, path: &str) -> Result<(), String> {
        match load_texture(path).await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest);
                self.textures.insert(id.to_string(), texture);
                Ok(())
            }
            Err(e) => Err(format!("Failed to load {}: {}", path, e)),
        }
    }

    /// Get a texture by ID
    pub fn get(&self, id: &str) -> Option<&Texture2D> {
        self.textures.get(id).or(self.missing_texture.as_ref())
    }

    /// Draw a sprite at a position
    pub fn draw(&self, id: &str, x: f32, y: f32, scale: f32, color: Color) {
        if let Some(texture) = self.get(id) {
            draw_texture_ex(
                texture,
                x,
                y,
                color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(
                        texture.width() * scale,
                        texture.height() * scale,
                    )),
                    ..Default::default()
                },
            );
        }
    }

    /// Draw a sprite centered at a position
    pub fn draw_centered(&self, id: &str, x: f32, y: f32, scale: f32, color: Color) {
        if let Some(texture) = self.get(id) {
            let w = texture.width() * scale;
            let h = texture.height() * scale;
            draw_texture_ex(
                texture,
                x - w / 2.0,
                y - h / 2.0,
                color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(w, h)),
                    ..Default::default()
                },
            );
        }
    }
}

impl Default for SpriteManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Animation frame data
pub struct Animation {
    pub frames: Vec<String>,
    pub frame_duration: f32,
    current_frame: usize,
    elapsed: f32,
    pub looping: bool,
}

impl Animation {
    pub fn new(frames: Vec<String>, frame_duration: f32, looping: bool) -> Self {
        Self {
            frames,
            frame_duration,
            current_frame: 0,
            elapsed: 0.0,
            looping,
        }
    }

    /// Update animation timing
    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;

        if self.elapsed >= self.frame_duration {
            self.elapsed -= self.frame_duration;
            self.current_frame += 1;

            if self.current_frame >= self.frames.len() {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                }
            }
        }
    }

    /// Get current frame sprite ID
    pub fn current(&self) -> &str {
        &self.frames[self.current_frame]
    }

    /// Reset animation
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.elapsed = 0.0;
    }

    /// Is animation finished (for non-looping)?
    pub fn is_finished(&self) -> bool {
        !self.looping && self.current_frame >= self.frames.len() - 1
    }
}
