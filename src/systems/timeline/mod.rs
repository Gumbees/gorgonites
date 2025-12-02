//! Timeline and divergence systems
//!
//! Tracks how far the player's world has diverged from real history.

mod divergence;
mod tech_tree;

pub use divergence::*;
pub use tech_tree::*;

use crate::game::Era;
use serde::{Deserialize, Serialize};

/// The state of the player's alternate timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Current era
    pub era: Era,

    /// Current year (can be BCE/CE)
    pub year: i32,

    /// Divergence from our reality (0.0 - 100.0)
    pub divergence: f32,

    /// Major changes from real history
    pub alterations: Vec<HistoricalAlteration>,

    /// Tech tree state
    pub technology: TechState,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            era: Era::StoneAge,
            year: -10000,
            divergence: 0.0,
            alterations: Vec::new(),
            technology: TechState::default(),
        }
    }
}

impl Timeline {
    pub fn new(starting_era: Era) -> Self {
        let year = match starting_era {
            Era::StoneAge => -10000,
            Era::BronzeAge => -3000,
            Era::IronAge => -1200,
            Era::Medieval => 500,
            Era::Renaissance => 1400,
            Era::Industrial => 1700,
            Era::Modern => 1900,
            Era::Divergent => 2100,
        };

        Self {
            era: starting_era,
            year,
            ..Default::default()
        }
    }

    /// Advance time
    pub fn advance(&mut self, years: i32) {
        self.year += years;

        // Check for era transitions
        self.check_era_transition();
    }

    /// Add divergence (clamped 0-100)
    pub fn add_divergence(&mut self, amount: f32) {
        self.divergence = (self.divergence + amount).clamp(0.0, 100.0);

        // High divergence can trigger era shift to Divergent
        if self.divergence >= 80.0 && self.era != Era::Divergent {
            // Not automatic - but events become weirder
        }
    }

    /// Record a historical alteration
    pub fn record_alteration(&mut self, alteration: HistoricalAlteration) {
        self.add_divergence(alteration.divergence_impact);
        self.alterations.push(alteration);
    }

    fn check_era_transition(&mut self) {
        let new_era = match self.year {
            y if y < -3000 => Era::StoneAge,
            y if y < -1200 => Era::BronzeAge,
            y if y < 500 => Era::IronAge,
            y if y < 1400 => Era::Medieval,
            y if y < 1700 => Era::Renaissance,
            y if y < 1900 => Era::Industrial,
            _ => Era::Modern,
        };

        // Only advance era if moving forward (no regression from year alone)
        if self.era_order(new_era) > self.era_order(self.era) {
            self.era = new_era;
        }
    }

    fn era_order(&self, era: Era) -> u8 {
        match era {
            Era::StoneAge => 0,
            Era::BronzeAge => 1,
            Era::IronAge => 2,
            Era::Medieval => 3,
            Era::Renaissance => 4,
            Era::Industrial => 5,
            Era::Modern => 6,
            Era::Divergent => 7,
        }
    }

    /// Get divergence category for narrative purposes
    pub fn divergence_category(&self) -> DivergenceCategory {
        match self.divergence as u32 {
            0..=20 => DivergenceCategory::Familiar,
            21..=50 => DivergenceCategory::Altered,
            51..=80 => DivergenceCategory::Radical,
            _ => DivergenceCategory::Alien,
        }
    }
}

/// A specific change from real history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalAlteration {
    /// What changed
    pub description: String,

    /// What it would have been in our timeline
    pub original: String,

    /// When this happened
    pub year: i32,

    /// How much it affected divergence
    pub divergence_impact: f32,

    /// Cascading effects
    pub ripple_effects: Vec<String>,
}

/// How divergent the timeline is
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivergenceCategory {
    /// 0-20%: Minor changes, history recognizable
    Familiar,
    /// 21-50%: Major events different, some things recognizable
    Altered,
    /// 51-80%: Radically different, few parallels
    Radical,
    /// 81-100%: Completely alien civilization path
    Alien,
}

impl DivergenceCategory {
    pub fn description(&self) -> &'static str {
        match self {
            DivergenceCategory::Familiar => "History with footnotes",
            DivergenceCategory::Altered => "A world that could have been",
            DivergenceCategory::Radical => "Through the looking glass",
            DivergenceCategory::Alien => "What even is this timeline?",
        }
    }
}
