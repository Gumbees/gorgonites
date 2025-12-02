//! Diplomacy system

use serde::{Deserialize, Serialize};

/// Current diplomatic state between two factions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiplomaticState {
    /// No contact yet
    Unknown,
    /// Neutral, can trade
    Neutral,
    /// Actively trading, open borders
    Friendly,
    /// Formal alliance
    Allied,
    /// Cold relations, trade restricted
    Hostile,
    /// Active warfare
    War,
    /// One is vassal of other
    Vassal,
}

/// A diplomatic action that can be proposed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiplomaticAction {
    /// Declare war
    DeclareWar,
    /// Propose peace
    ProposePeace { terms: PeaceTerms },
    /// Propose alliance
    ProposeAlliance,
    /// Break alliance
    BreakAlliance,
    /// Open trade
    OpenTrade,
    /// Close borders
    CloseBorders,
    /// Demand tribute
    DemandTribute { amount: u32 },
    /// Offer gift
    OfferGift { resource: String, amount: u32 },
    /// Demand vassalage
    DemandVassalage,
    /// Offer vassalage
    OfferVassalage,
}

/// Terms for peace negotiations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceTerms {
    /// Territory to cede
    pub territory_ceded: Vec<String>,
    /// Resources to pay
    pub reparations: u32,
    /// Hostages/guarantees
    pub hostages: bool,
    /// Non-aggression pact duration
    pub truce_duration: u32,
}

impl Default for PeaceTerms {
    fn default() -> Self {
        Self {
            territory_ceded: Vec::new(),
            reparations: 0,
            hostages: false,
            truce_duration: 10,
        }
    }
}

/// Calculates if an AI would accept a diplomatic proposal
pub fn calculate_acceptance(
    proposer_power: f32,
    target_power: f32,
    current_relation: i32,
    action: &DiplomaticAction,
) -> f32 {
    let power_ratio = proposer_power / target_power.max(0.1);
    let relation_factor = (current_relation as f32 + 100.0) / 200.0; // 0.0 to 1.0

    match action {
        DiplomaticAction::ProposeAlliance => {
            // More likely if similar power and good relations
            let power_similarity = 1.0 - (power_ratio - 1.0).abs().min(1.0);
            (power_similarity * 0.3 + relation_factor * 0.7) * 100.0
        }
        DiplomaticAction::ProposePeace { .. } => {
            // More likely if losing or exhausted
            if power_ratio > 1.0 {
                // They're stronger, we want peace
                70.0 + relation_factor * 30.0
            } else {
                // We're stronger, they want peace less
                30.0 + relation_factor * 20.0
            }
        }
        DiplomaticAction::DemandTribute { .. } => {
            // Only accept if much weaker
            if power_ratio > 1.5 {
                (power_ratio - 1.0) * 40.0
            } else {
                0.0
            }
        }
        DiplomaticAction::OfferGift { .. } => {
            // Usually accept gifts
            80.0 + relation_factor * 20.0
        }
        _ => relation_factor * 50.0,
    }
}

/// A treaty between factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treaty {
    pub id: String,
    pub faction_a: String,
    pub faction_b: String,
    pub treaty_type: TreatyType,
    pub signed_turn: u32,
    pub expires_turn: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreatyType {
    Peace,
    Trade,
    OpenBorders,
    Alliance,
    NonAggression,
    Vassalage,
}
