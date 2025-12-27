//! Sprite description generator using Ollama

use crate::ai::{OllamaClient, OllamaError};
use crate::config::OllamaConfig;
use rand::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// A generated sprite description
#[derive(Debug, Clone)]
pub struct SpriteDescription {
    /// Primary color (RGB)
    pub primary_color: [u8; 3],
    /// Secondary color (RGB)
    pub secondary_color: [u8; 3],
    /// Accent color (RGB)
    pub accent_color: [u8; 3],
    /// Shape type
    pub shape: SpriteShape,
    /// Size in pixels (width, height)
    pub size: (u8, u8),
    /// Pattern type
    pub pattern: SpritePattern,
    /// Creature name/type
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpriteShape {
    Blob,
    Humanoid,
    Quadruped,
    Flying,
    Serpent,
    Geometric,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpritePattern {
    Solid,
    Striped,
    Spotted,
    Gradient,
    Checkered,
}

impl Default for SpriteDescription {
    fn default() -> Self {
        Self {
            primary_color: [100, 150, 200],
            secondary_color: [50, 100, 150],
            accent_color: [255, 200, 100],
            shape: SpriteShape::Blob,
            size: (16, 16),
            pattern: SpritePattern::Solid,
            name: "Unknown Creature".to_string(),
        }
    }
}

/// Message types for async sprite generation
pub enum SpriteMessage {
    Generate { x: f32, y: f32 },
    Result(SpriteDescription, f32, f32),
    Error(String, f32, f32),
}

/// Manages async sprite generation via Ollama
pub struct SpriteGenerator {
    sender: Sender<SpriteMessage>,
    receiver: Receiver<SpriteMessage>,
    pending_generations: usize,
    rng: StdRng,
}

impl SpriteGenerator {
    /// Create a new sprite generator
    pub fn new(config: &OllamaConfig) -> Self {
        let (tx_to_worker, rx_from_main) = channel::<SpriteMessage>();
        let (tx_to_main, rx_from_worker) = channel::<SpriteMessage>();

        let config = config.clone();

        // Spawn worker thread for Ollama requests
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            while let Ok(msg) = rx_from_main.recv() {
                match msg {
                    SpriteMessage::Generate { x, y } => {
                        let result = rt.block_on(generate_sprite_async(&config, x, y));
                        match result {
                            Ok(desc) => {
                                let _ = tx_to_main.send(SpriteMessage::Result(desc, x, y));
                            }
                            Err(e) => {
                                let _ = tx_to_main.send(SpriteMessage::Error(e.to_string(), x, y));
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        Self {
            sender: tx_to_worker,
            receiver: rx_from_worker,
            pending_generations: 0,
            rng: StdRng::from_entropy(),
        }
    }

    /// Request a new sprite at the given position
    pub fn request_sprite(&mut self, x: f32, y: f32) {
        // Limit concurrent generations
        if self.pending_generations < 3 {
            let _ = self.sender.send(SpriteMessage::Generate { x, y });
            self.pending_generations += 1;
        }
    }

    /// Poll for completed sprites (non-blocking)
    pub fn poll(&mut self) -> Option<(SpriteDescription, f32, f32)> {
        match self.receiver.try_recv() {
            Ok(SpriteMessage::Result(desc, x, y)) => {
                self.pending_generations = self.pending_generations.saturating_sub(1);
                Some((desc, x, y))
            }
            Ok(SpriteMessage::Error(e, x, y)) => {
                self.pending_generations = self.pending_generations.saturating_sub(1);
                tracing::warn!("Sprite generation error: {}", e);
                // Return a fallback sprite at the original position
                Some((self.generate_fallback(), x, y))
            }
            _ => None,
        }
    }

    /// Generate a random fallback sprite (no AI)
    pub fn generate_fallback(&mut self) -> SpriteDescription {
        let shapes = [
            SpriteShape::Blob,
            SpriteShape::Humanoid,
            SpriteShape::Quadruped,
            SpriteShape::Flying,
            SpriteShape::Serpent,
            SpriteShape::Geometric,
        ];
        let patterns = [
            SpritePattern::Solid,
            SpritePattern::Striped,
            SpritePattern::Spotted,
            SpritePattern::Gradient,
        ];

        SpriteDescription {
            primary_color: [
                self.rng.gen_range(50..255),
                self.rng.gen_range(50..255),
                self.rng.gen_range(50..255),
            ],
            secondary_color: [
                self.rng.gen_range(30..200),
                self.rng.gen_range(30..200),
                self.rng.gen_range(30..200),
            ],
            accent_color: [
                self.rng.gen_range(150..255),
                self.rng.gen_range(150..255),
                self.rng.gen_range(50..150),
            ],
            shape: *shapes.choose(&mut self.rng).unwrap(),
            size: (
                self.rng.gen_range(12..24),
                self.rng.gen_range(12..24),
            ),
            pattern: *patterns.choose(&mut self.rng).unwrap(),
            name: generate_random_name(&mut self.rng),
        }
    }

    /// Check if there are pending generations
    pub fn is_busy(&self) -> bool {
        self.pending_generations > 0
    }
}

/// Generate a sprite description using Ollama
async fn generate_sprite_async(
    config: &OllamaConfig,
    _x: f32,
    _y: f32,
) -> Result<SpriteDescription, OllamaError> {
    let client = OllamaClient::new(config)?;

    // Check if Ollama is available
    if !client.health_check().await? {
        return Err(OllamaError::ConnectionFailed("Ollama not available".into()));
    }

    let prompt = r#"Generate a fantasy creature sprite description. Respond with ONLY these fields, one per line:
NAME: [creative creature name]
PRIMARY: [R,G,B] (main body color, values 0-255)
SECONDARY: [R,G,B] (accent color)
ACCENT: [R,G,B] (highlight color)
SHAPE: [blob/humanoid/quadruped/flying/serpent/geometric]
PATTERN: [solid/striped/spotted/gradient]
SIZE: [width,height] (8-32 pixels)

Be creative but keep response minimal."#;

    let response = client.generate(prompt, Some("You are a fantasy creature designer. Give brief, structured responses.")).await?;

    // Parse the response
    parse_sprite_response(&response)
}

/// Parse Ollama's response into a SpriteDescription
fn parse_sprite_response(response: &str) -> Result<SpriteDescription, OllamaError> {
    let mut desc = SpriteDescription::default();
    let mut rng = rand::thread_rng();

    for line in response.lines() {
        let line = line.trim();

        if let Some(name) = line.strip_prefix("NAME:") {
            desc.name = name.trim().to_string();
        } else if let Some(color) = line.strip_prefix("PRIMARY:") {
            if let Some(rgb) = parse_rgb(color) {
                desc.primary_color = rgb;
            }
        } else if let Some(color) = line.strip_prefix("SECONDARY:") {
            if let Some(rgb) = parse_rgb(color) {
                desc.secondary_color = rgb;
            }
        } else if let Some(color) = line.strip_prefix("ACCENT:") {
            if let Some(rgb) = parse_rgb(color) {
                desc.accent_color = rgb;
            }
        } else if let Some(shape) = line.strip_prefix("SHAPE:") {
            desc.shape = match shape.trim().to_lowercase().as_str() {
                "blob" => SpriteShape::Blob,
                "humanoid" => SpriteShape::Humanoid,
                "quadruped" => SpriteShape::Quadruped,
                "flying" => SpriteShape::Flying,
                "serpent" => SpriteShape::Serpent,
                "geometric" => SpriteShape::Geometric,
                _ => SpriteShape::Blob,
            };
        } else if let Some(pattern) = line.strip_prefix("PATTERN:") {
            desc.pattern = match pattern.trim().to_lowercase().as_str() {
                "solid" => SpritePattern::Solid,
                "striped" => SpritePattern::Striped,
                "spotted" => SpritePattern::Spotted,
                "gradient" => SpritePattern::Gradient,
                "checkered" => SpritePattern::Checkered,
                _ => SpritePattern::Solid,
            };
        } else if let Some(size) = line.strip_prefix("SIZE:") {
            if let Some((w, h)) = parse_size(size) {
                desc.size = (w, h);
            }
        }
    }

    // If name is still default, generate a random one
    if desc.name == "Unknown Creature" {
        desc.name = generate_random_name(&mut rng);
    }

    Ok(desc)
}

/// Parse RGB from string like "[255, 128, 64]" or "255,128,64"
fn parse_rgb(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_matches(|c| c == '[' || c == ']');
    let parts: Vec<&str> = s.split(',').collect();

    if parts.len() >= 3 {
        let r = parts[0].trim().parse::<u8>().ok()?;
        let g = parts[1].trim().parse::<u8>().ok()?;
        let b = parts[2].trim().parse::<u8>().ok()?;
        Some([r, g, b])
    } else {
        None
    }
}

/// Parse size from string like "[16, 16]" or "16,16"
fn parse_size(s: &str) -> Option<(u8, u8)> {
    let s = s.trim().trim_matches(|c| c == '[' || c == ']');
    let parts: Vec<&str> = s.split(',').collect();

    if parts.len() >= 2 {
        let w = parts[0].trim().parse::<u8>().ok()?.clamp(8, 32);
        let h = parts[1].trim().parse::<u8>().ok()?.clamp(8, 32);
        Some((w, h))
    } else {
        None
    }
}

/// Generate a random creature name
fn generate_random_name(rng: &mut impl Rng) -> String {
    let prefixes = ["Zor", "Kra", "Mog", "Vex", "Nyx", "Glu", "Bor", "Fae", "Dra", "Sli"];
    let middles = ["an", "ith", "or", "ex", "um", "on", "al", "ix", "ar", ""];
    let suffixes = ["ling", "beast", "spawn", "kin", "mite", "wyrm", "shade", "wisp", "form", ""];

    format!(
        "{}{}{}",
        prefixes.choose(rng).unwrap(),
        middles.choose(rng).unwrap(),
        suffixes.choose(rng).unwrap()
    )
}
