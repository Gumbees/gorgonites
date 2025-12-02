//! Combat system

use serde::{Deserialize, Serialize};

/// Damage types that can be dealt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    /// Physical melee damage
    Melee,
    /// Physical ranged damage (arrows, thrown weapons)
    Ranged,
    /// Siege damage (effective vs buildings)
    Siege,
    /// Fire damage
    Fire,
    /// Special/magical (for divergent timeline scenarios)
    Special,
}

/// Armor types that reduce damage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmorType {
    Unarmored,
    Light,
    Medium,
    Heavy,
    Building,
}

/// Calculate damage after armor reduction
pub fn calculate_damage(base_damage: f32, damage_type: DamageType, armor_type: ArmorType) -> f32 {
    let multiplier = match (damage_type, armor_type) {
        // Melee effectiveness
        (DamageType::Melee, ArmorType::Unarmored) => 1.0,
        (DamageType::Melee, ArmorType::Light) => 0.9,
        (DamageType::Melee, ArmorType::Medium) => 0.7,
        (DamageType::Melee, ArmorType::Heavy) => 0.5,
        (DamageType::Melee, ArmorType::Building) => 0.25,

        // Ranged effectiveness
        (DamageType::Ranged, ArmorType::Unarmored) => 1.2,
        (DamageType::Ranged, ArmorType::Light) => 1.0,
        (DamageType::Ranged, ArmorType::Medium) => 0.6,
        (DamageType::Ranged, ArmorType::Heavy) => 0.3,
        (DamageType::Ranged, ArmorType::Building) => 0.1,

        // Siege effectiveness
        (DamageType::Siege, ArmorType::Building) => 2.0,
        (DamageType::Siege, _) => 0.5,

        // Fire effectiveness
        (DamageType::Fire, ArmorType::Building) => 1.5,
        (DamageType::Fire, _) => 1.0,

        // Special ignores armor
        (DamageType::Special, _) => 1.0,
    };

    base_damage * multiplier
}

/// Result of a combat calculation
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub damage_dealt: f32,
    pub target_killed: bool,
    pub experience_gained: u32,
}
