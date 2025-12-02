//! Narrative events that occur during gameplay

use serde::{Deserialize, Serialize};
use crate::game::Era;

/// A narrative event presented to the player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEvent {
    /// Unique identifier
    pub id: String,

    /// Event title
    pub title: String,

    /// The narrative text (AI-generated or templated)
    pub description: String,

    /// Era this event is appropriate for
    pub era: Era,

    /// Available choices
    pub choices: Vec<EventChoice>,

    /// Is this event blocking? (game pauses until resolved)
    pub blocking: bool,

    /// Priority for display (higher = more urgent)
    pub priority: u8,
}

/// A choice the player can make in response to an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChoice {
    /// Choice identifier
    pub id: String,

    /// Text shown to player
    pub text: String,

    /// Hint about consequences (optional, may be hidden)
    pub hint: Option<String>,

    /// Effects of choosing this option
    pub effects: Vec<EventEffect>,

    /// Requirements to unlock this choice
    pub requirements: Vec<ChoiceRequirement>,
}

/// Effects that result from a choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventEffect {
    /// Change resources
    ModifyResource { resource: String, amount: i32 },

    /// Change divergence score
    ModifyDivergence { amount: f32 },

    /// Unlock or lock technology
    ModifyTech { tech_id: String, unlocked: bool },

    /// Change faction relations
    ModifyRelation { faction: String, amount: i32 },

    /// Spawn or despawn units
    SpawnUnits { unit_type: String, count: u32 },

    /// Trigger another event
    TriggerEvent { event_id: String, delay: f32 },

    /// Add a character to the story
    IntroduceCharacter { character_id: String },

    /// Custom effect handled by game logic
    Custom { effect_type: String, data: String },
}

/// Requirements to make a choice available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChoiceRequirement {
    /// Must have at least this much of a resource
    HasResource { resource: String, amount: u32 },

    /// Must have unlocked a technology
    HasTech { tech_id: String },

    /// Divergence must be within range
    DivergenceRange { min: f32, max: f32 },

    /// Must have a specific character
    HasCharacter { character_id: String },

    /// Custom requirement
    Custom { requirement_type: String, data: String },
}

/// Categories of events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    /// Major historical turning point
    Historical,
    /// Conflict and warfare
    Military,
    /// Trade and economy
    Economic,
    /// Science and technology
    Scientific,
    /// Religion and culture
    Cultural,
    /// Natural disasters
    Natural,
    /// Character-driven drama
    Personal,
    /// Completely AI-generated
    Generated,
}
