//! Historical era definitions and progression

use serde::{Deserialize, Serialize};

/// Historical eras that define technology, aesthetics, and available units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Era {
    /// ~3M BCE - 3000 BCE: Tribal survival, stone tools, fire
    StoneAge,

    /// ~3000 BCE - 1200 BCE: Early empires, bronze weapons, chariots, writing
    BronzeAge,

    /// ~1200 BCE - 500 CE: Classical civilizations, iron weapons, philosophy
    IronAge,

    /// ~500 CE - 1400 CE: Feudalism, castles, knights, religious conflict
    Medieval,

    /// ~1400 CE - 1700 CE: Gunpowder, exploration, printing, scientific revolution
    Renaissance,

    /// ~1700 CE - 1900 CE: Factories, railways, nationalism, colonial empires
    Industrial,

    /// ~1900 CE - Present: Total war, nuclear age, information technology
    Modern,

    /// Beyond our timeline - player has diverged into unknown territory
    Divergent,
}

impl Era {
    /// Get the next era in normal progression
    pub fn next(&self) -> Option<Era> {
        match self {
            Era::StoneAge => Some(Era::BronzeAge),
            Era::BronzeAge => Some(Era::IronAge),
            Era::IronAge => Some(Era::Medieval),
            Era::Medieval => Some(Era::Renaissance),
            Era::Renaissance => Some(Era::Industrial),
            Era::Industrial => Some(Era::Modern),
            Era::Modern => Some(Era::Divergent),
            Era::Divergent => None,
        }
    }

    /// Get the previous era
    pub fn previous(&self) -> Option<Era> {
        match self {
            Era::StoneAge => None,
            Era::BronzeAge => Some(Era::StoneAge),
            Era::IronAge => Some(Era::BronzeAge),
            Era::Medieval => Some(Era::IronAge),
            Era::Renaissance => Some(Era::Medieval),
            Era::Industrial => Some(Era::Renaissance),
            Era::Modern => Some(Era::Industrial),
            Era::Divergent => Some(Era::Modern),
        }
    }

    /// Get era name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            Era::StoneAge => "Stone Age",
            Era::BronzeAge => "Bronze Age",
            Era::IronAge => "Iron Age",
            Era::Medieval => "Medieval",
            Era::Renaissance => "Renaissance",
            Era::Industrial => "Industrial Age",
            Era::Modern => "Modern Age",
            Era::Divergent => "Divergent Timeline",
        }
    }

    /// Get approximate year range for display
    pub fn year_range(&self) -> &'static str {
        match self {
            Era::StoneAge => "~3,000,000 BCE - 3,000 BCE",
            Era::BronzeAge => "~3,000 BCE - 1,200 BCE",
            Era::IronAge => "~1,200 BCE - 500 CE",
            Era::Medieval => "~500 CE - 1,400 CE",
            Era::Renaissance => "~1,400 CE - 1,700 CE",
            Era::Industrial => "~1,700 CE - 1,900 CE",
            Era::Modern => "~1,900 CE - Present",
            Era::Divergent => "Unknown",
        }
    }

    /// Key technologies/concepts unlocked in this era
    pub fn key_technologies(&self) -> &'static [&'static str] {
        match self {
            Era::StoneAge => &["Fire", "Stone Tools", "Language", "Tribal Organization"],
            Era::BronzeAge => &["Bronze Working", "Writing", "Wheel", "Organized Religion", "City-States"],
            Era::IronAge => &["Iron Working", "Coinage", "Philosophy", "Democracy", "Empires"],
            Era::Medieval => &["Feudalism", "Castles", "Heavy Cavalry", "Universities", "Guilds"],
            Era::Renaissance => &["Gunpowder", "Printing Press", "Banking", "Navigation", "Scientific Method"],
            Era::Industrial => &["Steam Power", "Railways", "Factories", "Telegraph", "Mass Production"],
            Era::Modern => &["Electricity", "Nuclear Power", "Computers", "Internet", "Space Travel"],
            Era::Divergent => &["???"],
        }
    }
}

impl Default for Era {
    fn default() -> Self {
        Era::StoneAge
    }
}

/// Requirements to advance to the next era
#[derive(Debug, Clone)]
pub struct EraTransition {
    pub from: Era,
    pub to: Era,
    pub required_techs: Vec<String>,
    pub required_resources: Vec<(String, u32)>,
    pub narrative_trigger: Option<String>,
}
