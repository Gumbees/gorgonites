//! Entity definitions

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Unique identifier for an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

/// An entity is a collection of components
pub struct Entity {
    pub id: EntityId,
    components: HashMap<TypeId, Box<dyn Any>>,
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            components: HashMap::new(),
        }
    }

    /// Add a component to this entity
    pub fn add<T: 'static>(&mut self, component: T) {
        self.components.insert(TypeId::of::<T>(), Box::new(component));
    }

    /// Remove a component from this entity
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.components
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast().ok())
            .map(|boxed| *boxed)
    }

    /// Get a reference to a component
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    /// Get a mutable reference to a component
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut())
    }

    /// Check if entity has a component
    pub fn has<T: 'static>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }
}
