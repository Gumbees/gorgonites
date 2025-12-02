//! Technology tree with branching paths

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::game::Era;

/// The state of a player's technology
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechState {
    /// Technologies that have been researched
    pub unlocked: HashSet<String>,

    /// Technologies currently being researched
    pub in_progress: HashMap<String, f32>,

    /// Technologies that are blocked (cannot be researched)
    pub blocked: HashSet<String>,

    /// Research points per turn
    pub research_rate: f32,
}

impl TechState {
    pub fn new() -> Self {
        Self {
            unlocked: HashSet::new(),
            in_progress: HashMap::new(),
            blocked: HashSet::new(),
            research_rate: 1.0,
        }
    }

    /// Check if a technology is unlocked
    pub fn has(&self, tech_id: &str) -> bool {
        self.unlocked.contains(tech_id)
    }

    /// Unlock a technology
    pub fn unlock(&mut self, tech_id: &str) {
        self.in_progress.remove(tech_id);
        self.unlocked.insert(tech_id.to_string());
    }

    /// Block a technology (choice-based locking)
    pub fn block(&mut self, tech_id: &str) {
        self.in_progress.remove(tech_id);
        self.blocked.insert(tech_id.to_string());
    }
}

/// Definition of a technology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Technology {
    /// Unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Era this technology belongs to
    pub era: Era,

    /// Research cost
    pub cost: f32,

    /// Prerequisites (AND - all required)
    pub requires: Vec<String>,

    /// Mutually exclusive with (OR - any blocks this)
    pub excludes: Vec<String>,

    /// What this technology enables
    pub unlocks: Vec<TechUnlock>,

    /// How much this tech diverges from real history if unlocked early/differently
    pub divergence_weight: f32,
}

/// What a technology unlocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechUnlock {
    /// Unlocks a unit type
    Unit(String),

    /// Unlocks a building type
    Building(String),

    /// Unlocks a resource type
    Resource(String),

    /// Enables an ability
    Ability(String),

    /// Modifies a stat
    Modifier { stat: String, value: f32 },

    /// Unlocks another tech for research
    Technology(String),
}

/// Example tech tree builder
pub fn build_stone_age_techs() -> Vec<Technology> {
    vec![
        Technology {
            id: "fire".to_string(),
            name: "Fire".to_string(),
            description: "Harness the power of flame for warmth, cooking, and protection.".to_string(),
            era: Era::StoneAge,
            cost: 10.0,
            requires: vec![],
            excludes: vec![],
            unlocks: vec![
                TechUnlock::Building("campfire".to_string()),
                TechUnlock::Ability("cook_food".to_string()),
            ],
            divergence_weight: 0.0, // Expected
        },
        Technology {
            id: "stone_tools".to_string(),
            name: "Stone Tools".to_string(),
            description: "Shape stone into useful implements.".to_string(),
            era: Era::StoneAge,
            cost: 15.0,
            requires: vec![],
            excludes: vec![],
            unlocks: vec![
                TechUnlock::Unit("gatherer".to_string()),
                TechUnlock::Modifier { stat: "gather_rate".to_string(), value: 1.5 },
            ],
            divergence_weight: 0.0,
        },
        Technology {
            id: "language".to_string(),
            name: "Language".to_string(),
            description: "Develop complex communication.".to_string(),
            era: Era::StoneAge,
            cost: 20.0,
            requires: vec![],
            excludes: vec![],
            unlocks: vec![
                TechUnlock::Ability("trade".to_string()),
                TechUnlock::Technology("oral_tradition".to_string()),
            ],
            divergence_weight: 0.0,
        },
        Technology {
            id: "agriculture".to_string(),
            name: "Agriculture".to_string(),
            description: "Domesticate plants for reliable food.".to_string(),
            era: Era::StoneAge,
            cost: 50.0,
            requires: vec!["stone_tools".to_string()],
            excludes: vec!["nomadic_mastery".to_string()],
            unlocks: vec![
                TechUnlock::Building("farm".to_string()),
                TechUnlock::Resource("grain".to_string()),
            ],
            divergence_weight: 5.0, // Choosing differently has long-term effects
        },
        Technology {
            id: "nomadic_mastery".to_string(),
            name: "Nomadic Mastery".to_string(),
            description: "Perfect the art of mobile living.".to_string(),
            era: Era::StoneAge,
            cost: 50.0,
            requires: vec!["stone_tools".to_string()],
            excludes: vec!["agriculture".to_string()],
            unlocks: vec![
                TechUnlock::Unit("scout".to_string()),
                TechUnlock::Modifier { stat: "movement_speed".to_string(), value: 1.3 },
            ],
            divergence_weight: 10.0, // Less common path
        },
    ]
}
