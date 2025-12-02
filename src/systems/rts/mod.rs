//! Real-Time Strategy systems
//!
//! Handles unit control, resource management, combat, and base building.

mod units;
mod resources;
mod combat;

pub use units::*;
pub use resources::*;
pub use combat::*;

/// Configuration for RTS gameplay
pub struct RtsConfig {
    /// Base movement speed multiplier
    pub speed_multiplier: f32,

    /// Fog of war enabled
    pub fog_of_war: bool,

    /// Maximum units per player
    pub unit_cap: u32,
}

impl Default for RtsConfig {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            fog_of_war: true,
            unit_cap: 200,
        }
    }
}
