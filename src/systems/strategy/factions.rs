//! Faction definitions and management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::game::Era;

/// A faction/civilization in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    /// Unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Faction color for map/units
    pub color: [u8; 4],

    /// Is this player-controlled?
    pub is_player: bool,

    /// AI personality (if AI-controlled)
    pub ai_personality: Option<AiPersonality>,

    /// Current era
    pub era: Era,

    /// Unique bonuses/traits
    pub traits: Vec<FactionTrait>,

    /// Relations with other factions
    pub relations: HashMap<String, i32>,

    /// Territory controlled
    pub territory_count: u32,

    /// Population
    pub population: u64,
}

impl Faction {
    pub fn new_player(id: &str, name: &str, color: [u8; 4]) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            color,
            is_player: true,
            ai_personality: None,
            era: Era::StoneAge,
            traits: Vec::new(),
            relations: HashMap::new(),
            territory_count: 1,
            population: 100,
        }
    }

    pub fn new_ai(id: &str, name: &str, color: [u8; 4], personality: AiPersonality) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            color,
            is_player: false,
            ai_personality: Some(personality),
            era: Era::StoneAge,
            traits: Vec::new(),
            relations: HashMap::new(),
            territory_count: 1,
            population: 100,
        }
    }

    /// Get relation with another faction
    pub fn relation_with(&self, other_id: &str) -> i32 {
        *self.relations.get(other_id).unwrap_or(&0)
    }

    /// Modify relation with another faction
    pub fn modify_relation(&mut self, other_id: &str, delta: i32) {
        let current = self.relations.entry(other_id.to_string()).or_insert(0);
        *current = (*current + delta).clamp(-100, 100);
    }
}

/// AI behavior personality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiPersonality {
    /// Prefers military solutions
    Aggressive,
    /// Prefers diplomacy and trade
    Diplomatic,
    /// Focuses on expansion
    Expansionist,
    /// Focuses on technology
    Scientific,
    /// Focuses on culture and religion
    Cultural,
    /// Defensive, turtles up
    Isolationist,
    /// Unpredictable, changes strategy
    Chaotic,
}

/// Special traits that define a faction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionTrait {
    pub id: String,
    pub name: String,
    pub description: String,
    pub modifiers: Vec<FactionModifier>,
}

/// Modifiers from faction traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactionModifier {
    /// Bonus to resource gathering
    GatherRate { resource: String, multiplier: f32 },

    /// Bonus to unit stats
    UnitStat { stat: String, multiplier: f32 },

    /// Bonus to research
    ResearchRate(f32),

    /// Bonus to diplomacy
    DiplomacyBonus(i32),

    /// Unique unit access
    UniqueUnit(String),

    /// Unique building access
    UniqueBuilding(String),
}

/// Predefined faction templates
pub fn default_factions() -> Vec<Faction> {
    vec![
        Faction::new_ai(
            "tribe_of_the_sun",
            "Tribe of the Sun",
            [255, 200, 50, 255],
            AiPersonality::Expansionist,
        ),
        Faction::new_ai(
            "river_people",
            "River People",
            [50, 150, 255, 255],
            AiPersonality::Diplomatic,
        ),
        Faction::new_ai(
            "mountain_clans",
            "Mountain Clans",
            [150, 150, 150, 255],
            AiPersonality::Isolationist,
        ),
        Faction::new_ai(
            "forest_dwellers",
            "Forest Dwellers",
            [50, 200, 50, 255],
            AiPersonality::Scientific,
        ),
    ]
}
