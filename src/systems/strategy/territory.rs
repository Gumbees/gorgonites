//! Territory and map control

use serde::{Deserialize, Serialize};

/// A region on the map that can be controlled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Territory {
    /// Unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Owning faction (None if unclaimed)
    pub owner: Option<String>,

    /// Map position (center)
    pub position: (f32, f32),

    /// Size/importance
    pub size: TerritorySize,

    /// Terrain type
    pub terrain: TerrainType,

    /// Resources available
    pub resources: Vec<TerritoryResource>,

    /// Population capacity
    pub population_cap: u32,

    /// Current population
    pub population: u32,

    /// Development level (0.0 - 1.0)
    pub development: f32,

    /// Fortification level
    pub fortification: u8,

    /// Adjacent territory IDs
    pub neighbors: Vec<String>,
}

impl Territory {
    pub fn is_contested(&self) -> bool {
        // Could check for enemy units present
        false
    }

    pub fn production_value(&self) -> f32 {
        self.development * self.size.multiplier() * (self.population as f32 / 100.0)
    }
}

/// Size categories for territories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerritorySize {
    Outpost,    // Small, limited resources
    Settlement, // Medium, can grow
    City,       // Large, major production
    Capital,    // Faction capital, very important
}

impl TerritorySize {
    pub fn multiplier(&self) -> f32 {
        match self {
            TerritorySize::Outpost => 0.5,
            TerritorySize::Settlement => 1.0,
            TerritorySize::City => 2.0,
            TerritorySize::Capital => 3.0,
        }
    }

    pub fn population_cap(&self) -> u32 {
        match self {
            TerritorySize::Outpost => 100,
            TerritorySize::Settlement => 500,
            TerritorySize::City => 2000,
            TerritorySize::Capital => 5000,
        }
    }
}

/// Terrain types affecting movement and production
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Plains,
    Hills,
    Mountains,
    Forest,
    Desert,
    Tundra,
    Swamp,
    Coastal,
    River,
}

impl TerrainType {
    pub fn movement_cost(&self) -> f32 {
        match self {
            TerrainType::Plains => 1.0,
            TerrainType::Hills => 1.5,
            TerrainType::Mountains => 3.0,
            TerrainType::Forest => 1.5,
            TerrainType::Desert => 2.0,
            TerrainType::Tundra => 2.0,
            TerrainType::Swamp => 2.5,
            TerrainType::Coastal => 1.0,
            TerrainType::River => 0.5, // Rivers help movement
        }
    }

    pub fn defense_bonus(&self) -> f32 {
        match self {
            TerrainType::Plains => 0.0,
            TerrainType::Hills => 0.25,
            TerrainType::Mountains => 0.5,
            TerrainType::Forest => 0.25,
            TerrainType::Desert => 0.0,
            TerrainType::Tundra => 0.0,
            TerrainType::Swamp => 0.1,
            TerrainType::Coastal => 0.0,
            TerrainType::River => 0.15,
        }
    }
}

/// Resources available in a territory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryResource {
    pub resource_type: String,
    pub abundance: f32, // 0.0 - 1.0
    pub depleted: bool,
}

/// Victory conditions for a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VictoryCondition {
    /// Control X% of all territory
    Domination { percentage: f32 },

    /// Achieve maximum divergence
    Divergence { target: f32 },

    /// Survive to a specific year
    Survival { target_year: i32 },

    /// Complete a specific narrative arc
    Story { arc_id: String },

    /// Achieve all other victory conditions
    Total,
}

impl VictoryCondition {
    pub fn description(&self) -> String {
        match self {
            VictoryCondition::Domination { percentage } => {
                format!("Control {}% of all territory", percentage)
            }
            VictoryCondition::Divergence { target } => {
                format!("Reach {}% timeline divergence", target)
            }
            VictoryCondition::Survival { target_year } => {
                format!("Survive until the year {}", target_year)
            }
            VictoryCondition::Story { arc_id } => {
                format!("Complete the {} story arc", arc_id)
            }
            VictoryCondition::Total => "Achieve total victory".to_string(),
        }
    }
}
