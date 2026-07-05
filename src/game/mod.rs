//! The battlefield simulation core.
//!
//! Everything in this module is engine-agnostic: it depends on math types
//! only, never on rendering, windowing, or input. The Bevy frontend
//! (`crate::frontend`) drives it and draws it.
//!
//! Gameplay is Rise of Nations: national borders, attrition, commerce-capped
//! economy, age advancement, city capture, capital-loss countdown.

mod ai_nation;
mod entities;
mod era;
mod mapgen;
mod world;

pub use entities::*;
pub use era::*;
pub use mapgen::{hash01, GameMap, Terrain, MAP_H, MAP_W, TILE};
pub use world::{
    age_up_cost, Nation, Particle, ParticleKind, Rgb, Rgba, World, CAPITAL_COUNTDOWN,
};
