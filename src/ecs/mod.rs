//! Entity-Component-System implementation
//!
//! A simple ECS for managing game entities (units, buildings, resources, etc.)

mod entity;
mod component;
mod system;

pub use entity::*;
pub use component::*;
pub use system::*;

use std::collections::HashMap;
use uuid::Uuid;

/// The ECS world containing all entities and their components
pub struct EcsWorld {
    entities: HashMap<EntityId, Entity>,
    next_id: u64,
}

impl EcsWorld {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 0,
        }
    }

    /// Spawn a new entity and return its ID
    pub fn spawn(&mut self) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;

        self.entities.insert(id, Entity::new(id));
        id
    }

    /// Despawn an entity
    pub fn despawn(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id)
    }

    /// Get an entity by ID
    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// Get a mutable entity by ID
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    /// Iterate over all entities
    pub fn iter(&self) -> impl Iterator<Item = (&EntityId, &Entity)> {
        self.entities.iter()
    }

    /// Count of entities
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

impl Default for EcsWorld {
    fn default() -> Self {
        Self::new()
    }
}
