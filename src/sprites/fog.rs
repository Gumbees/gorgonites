//! 8-bit style fog particle system

use macroquad::prelude::*;
use ::rand::prelude::*;
use ::rand::rngs::StdRng;
use ::rand::Rng;

/// A single fog particle
#[derive(Clone)]
struct FogParticle {
    x: f32,
    y: f32,
    size: f32,
    alpha: f32,
    target_alpha: f32,
    lifetime: f32,
    max_lifetime: f32,
    drift_x: f32,
    drift_y: f32,
    pixel_pattern: Vec<Vec<bool>>,
}

impl FogParticle {
    fn new(x: f32, y: f32, rng: &mut impl Rng) -> Self {
        let size = rng.gen_range(16.0..32.0);
        let pattern_size = (size / 4.0) as usize;

        // Generate pixelated cloud pattern
        let mut pattern = vec![vec![false; pattern_size]; pattern_size];
        let center = pattern_size / 2;

        for py in 0..pattern_size {
            for px in 0..pattern_size {
                let dx = (px as i32 - center as i32).abs() as f32;
                let dy = (py as i32 - center as i32).abs() as f32;
                let dist = (dx * dx + dy * dy).sqrt();

                // Cloud-like shape with some randomness
                if dist < center as f32 + rng.gen_range(-1.0..1.0) {
                    pattern[py][px] = rng.gen_bool(0.7);
                }
            }
        }

        Self {
            x,
            y,
            size,
            alpha: 0.0,
            target_alpha: rng.gen_range(0.3..0.6),
            lifetime: 0.0,
            max_lifetime: rng.gen_range(2.0..4.0),
            drift_x: rng.gen_range(-15.0..15.0),
            drift_y: rng.gen_range(-20.0..-5.0), // Drift upward
            pixel_pattern: pattern,
        }
    }

    fn update(&mut self, dt: f32) {
        self.lifetime += dt;

        // Drift
        self.x += self.drift_x * dt;
        self.y += self.drift_y * dt;

        // Fade in/out based on lifetime
        let progress = self.lifetime / self.max_lifetime;
        if progress < 0.2 {
            // Fade in
            self.alpha = (progress / 0.2) * self.target_alpha;
        } else if progress > 0.7 {
            // Fade out
            self.alpha = ((1.0 - progress) / 0.3) * self.target_alpha;
        } else {
            self.alpha = self.target_alpha;
        }
    }

    fn is_dead(&self) -> bool {
        self.lifetime >= self.max_lifetime
    }

    fn render(&self) {
        let pixel_size = 4.0; // Each "pixel" in the fog
        let pattern_size = self.pixel_pattern.len();

        for (py, row) in self.pixel_pattern.iter().enumerate() {
            for (px, &filled) in row.iter().enumerate() {
                if filled {
                    let draw_x = self.x + px as f32 * pixel_size - (pattern_size as f32 * pixel_size / 2.0);
                    let draw_y = self.y + py as f32 * pixel_size - (pattern_size as f32 * pixel_size / 2.0);

                    // Slight color variation for depth
                    let shade = 0.8 + (py as f32 / pattern_size as f32) * 0.2;
                    let color = Color::new(
                        0.7 * shade,
                        0.75 * shade,
                        0.85 * shade,
                        self.alpha,
                    );

                    draw_rectangle(draw_x, draw_y, pixel_size, pixel_size, color);
                }
            }
        }
    }
}

/// Manages fog particles that appear around mouse movement
pub struct FogSystem {
    particles: Vec<FogParticle>,
    rng: StdRng,
    last_mouse_pos: (f32, f32),
    spawn_timer: f32,
    max_particles: usize,
}

impl FogSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            rng: StdRng::from_entropy(),
            last_mouse_pos: (0.0, 0.0),
            spawn_timer: 0.0,
            max_particles: 30,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let (mx, my) = mouse_position();

        // Calculate mouse velocity
        let (lx, ly) = self.last_mouse_pos;
        let velocity = ((mx - lx).powi(2) + (my - ly).powi(2)).sqrt();

        // Spawn new particles based on mouse movement
        self.spawn_timer += dt;
        if velocity > 5.0 && self.spawn_timer > 0.1 && self.particles.len() < self.max_particles {
            self.spawn_timer = 0.0;

            // Spawn fog particles trailing behind mouse
            let spawn_count = (velocity / 30.0).ceil() as usize;
            for i in 0..spawn_count.min(3) {
                let t = i as f32 / spawn_count.max(1) as f32;
                let spawn_x = lx + (mx - lx) * t + self.rng.gen_range(-20.0..20.0);
                let spawn_y = ly + (my - ly) * t + self.rng.gen_range(-20.0..20.0);

                self.particles.push(FogParticle::new(spawn_x, spawn_y, &mut self.rng));
            }
        }

        // Update existing particles
        for particle in &mut self.particles {
            particle.update(dt);
        }

        // Remove dead particles
        self.particles.retain(|p| !p.is_dead());

        self.last_mouse_pos = (mx, my);
    }

    pub fn render(&self) {
        for particle in &self.particles {
            particle.render();
        }
    }

    pub fn count(&self) -> usize {
        self.particles.len()
    }
}

impl Default for FogSystem {
    fn default() -> Self {
        Self::new()
    }
}
