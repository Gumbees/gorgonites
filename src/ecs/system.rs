//! System trait and common systems

use super::EcsWorld;

/// A system that operates on entities with specific components
pub trait System {
    /// Update this system
    fn update(&mut self, world: &mut EcsWorld, dt: f32);
}

/// Movement system - updates positions based on velocities
pub struct MovementSystem;

impl System for MovementSystem {
    fn update(&mut self, world: &mut EcsWorld, dt: f32) {
        use super::component::{Position, Velocity};

        for (_, entity) in world.iter() {
            if let (Some(pos), Some(vel)) = (entity.get::<Position>(), entity.get::<Velocity>()) {
                // Note: We'd need interior mutability here in a real implementation
                // For now this is a placeholder showing the pattern
                let _ = (pos, vel, dt);
            }
        }
    }
}

/// Combat system - handles attacks and damage
pub struct CombatSystem;

impl System for CombatSystem {
    fn update(&mut self, _world: &mut EcsWorld, _dt: f32) {
        // TODO: Implement combat logic
    }
}

/// Selection system - handles player unit selection
pub struct SelectionSystem;

impl System for SelectionSystem {
    fn update(&mut self, _world: &mut EcsWorld, _dt: f32) {
        // TODO: Handle mouse selection
    }
}
