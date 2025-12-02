//! Unit management and control

use serde::{Deserialize, Serialize};
use crate::game::Era;

/// Definition of a unit type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDefinition {
    pub id: String,
    pub name: String,
    pub era: Era,
    pub health: f32,
    pub speed: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
    pub cost: ResourceCost,
    pub build_time: f32,
}

/// Cost in resources to build something
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceCost {
    pub food: u32,
    pub wood: u32,
    pub stone: u32,
    pub metal: u32,
    pub gold: u32,
}

/// Command that can be issued to units
#[derive(Debug, Clone)]
pub enum UnitCommand {
    Move { x: f32, y: f32 },
    Attack { target_id: u64 },
    Patrol { points: Vec<(f32, f32)> },
    Guard { target_id: u64 },
    Gather { resource_id: u64 },
    Build { building_type: String, x: f32, y: f32 },
    Stop,
}

/// Unit AI state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitState {
    Idle,
    Moving,
    Attacking,
    Gathering,
    Building,
    Dead,
}
