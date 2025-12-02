//! Common component definitions

use serde::{Deserialize, Serialize};

/// Position in 2D world space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Velocity for moving entities
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Health for units and buildings
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn percentage(&self) -> f32 {
        self.current / self.max
    }
}

/// Selectable by player
#[derive(Debug, Clone, Copy, Default)]
pub struct Selectable {
    pub selected: bool,
}

/// Faction/team ownership
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Faction(pub u8);

impl Faction {
    pub const PLAYER: Faction = Faction(0);
    pub const NEUTRAL: Faction = Faction(255);
}

/// Render information
#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture_id: String,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
}

impl Sprite {
    pub fn colored(width: f32, height: f32, color: [f32; 4]) -> Self {
        Self {
            texture_id: String::new(),
            width,
            height,
            color,
        }
    }
}

/// Unit that can move and attack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub name: String,
    pub speed: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
}

/// Building that produces units or resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub name: String,
    pub produces: Vec<String>,
    pub production_time: f32,
}

/// Resource node (gold, wood, stone, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: String,
    pub amount: u32,
}
