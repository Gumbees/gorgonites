//! Resource management system

use serde::{Deserialize, Serialize};

/// Types of resources in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Food,
    Wood,
    Stone,
    Metal,
    Gold,
    /// Special resources that vary by era
    Special(u8),
}

impl ResourceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ResourceType::Food => "Food",
            ResourceType::Wood => "Wood",
            ResourceType::Stone => "Stone",
            ResourceType::Metal => "Metal",
            ResourceType::Gold => "Gold",
            ResourceType::Special(_) => "Special",
        }
    }
}

/// A player's resource stockpile
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceStockpile {
    pub food: u32,
    pub wood: u32,
    pub stone: u32,
    pub metal: u32,
    pub gold: u32,
}

impl ResourceStockpile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_starting_resources() -> Self {
        Self {
            food: 200,
            wood: 200,
            stone: 100,
            metal: 0,
            gold: 0,
        }
    }

    pub fn get(&self, resource_type: ResourceType) -> u32 {
        match resource_type {
            ResourceType::Food => self.food,
            ResourceType::Wood => self.wood,
            ResourceType::Stone => self.stone,
            ResourceType::Metal => self.metal,
            ResourceType::Gold => self.gold,
            ResourceType::Special(_) => 0,
        }
    }

    pub fn add(&mut self, resource_type: ResourceType, amount: u32) {
        match resource_type {
            ResourceType::Food => self.food += amount,
            ResourceType::Wood => self.wood += amount,
            ResourceType::Stone => self.stone += amount,
            ResourceType::Metal => self.metal += amount,
            ResourceType::Gold => self.gold += amount,
            ResourceType::Special(_) => {},
        }
    }

    pub fn spend(&mut self, resource_type: ResourceType, amount: u32) -> bool {
        let current = self.get(resource_type);
        if current >= amount {
            match resource_type {
                ResourceType::Food => self.food -= amount,
                ResourceType::Wood => self.wood -= amount,
                ResourceType::Stone => self.stone -= amount,
                ResourceType::Metal => self.metal -= amount,
                ResourceType::Gold => self.gold -= amount,
                ResourceType::Special(_) => return false,
            }
            true
        } else {
            false
        }
    }
}
