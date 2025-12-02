//! Divergence calculation and tracking

use serde::{Deserialize, Serialize};

/// Tracks what aspects of history have diverged
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DivergenceMetrics {
    /// Political divergence (borders, governments)
    pub political: f32,

    /// Technological divergence (inventions, timelines)
    pub technological: f32,

    /// Cultural divergence (religions, customs)
    pub cultural: f32,

    /// Economic divergence (trade, resources)
    pub economic: f32,

    /// Demographic divergence (populations, migrations)
    pub demographic: f32,
}

impl DivergenceMetrics {
    /// Calculate overall divergence
    pub fn total(&self) -> f32 {
        // Weighted average - some aspects count more
        let weighted = self.political * 0.25
            + self.technological * 0.25
            + self.cultural * 0.20
            + self.economic * 0.15
            + self.demographic * 0.15;

        weighted.clamp(0.0, 100.0)
    }

    /// Add divergence to a specific category
    pub fn add(&mut self, category: DivergenceType, amount: f32) {
        let target = match category {
            DivergenceType::Political => &mut self.political,
            DivergenceType::Technological => &mut self.technological,
            DivergenceType::Cultural => &mut self.cultural,
            DivergenceType::Economic => &mut self.economic,
            DivergenceType::Demographic => &mut self.demographic,
        };
        *target = (*target + amount).clamp(0.0, 100.0);
    }
}

/// Types of historical divergence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DivergenceType {
    Political,
    Technological,
    Cultural,
    Economic,
    Demographic,
}

/// "What if" scenario seeds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatIfScenario {
    /// Scenario name
    pub name: String,

    /// Description
    pub description: String,

    /// Starting divergence
    pub starting_divergence: f32,

    /// Which metrics start non-zero
    pub initial_metrics: DivergenceMetrics,

    /// Key altered events
    pub key_alterations: Vec<String>,
}

impl WhatIfScenario {
    /// Standard scenario - no prior divergence
    pub fn standard() -> Self {
        Self {
            name: "Our Timeline".to_string(),
            description: "History as we know it. Your choices write the future.".to_string(),
            starting_divergence: 0.0,
            initial_metrics: DivergenceMetrics::default(),
            key_alterations: vec![],
        }
    }

    /// Example: Rome never fell
    pub fn rome_eternal() -> Self {
        Self {
            name: "Rome Eternal".to_string(),
            description: "The Roman Empire never fell. It is now 2750 AUC.".to_string(),
            starting_divergence: 35.0,
            initial_metrics: DivergenceMetrics {
                political: 45.0,
                technological: 20.0,
                cultural: 40.0,
                economic: 30.0,
                demographic: 25.0,
            },
            key_alterations: vec![
                "The Crisis of the Third Century was resolved peacefully".to_string(),
                "Christianity never became the state religion".to_string(),
                "The Western Empire reformed before collapsing".to_string(),
            ],
        }
    }

    /// Example: China discovers America
    pub fn zheng_he_continues() -> Self {
        Self {
            name: "The Eastern Shore".to_string(),
            description: "Zheng He's voyages continued west. China discovered the Americas.".to_string(),
            starting_divergence: 40.0,
            initial_metrics: DivergenceMetrics {
                political: 35.0,
                technological: 25.0,
                cultural: 50.0,
                economic: 45.0,
                demographic: 40.0,
            },
            key_alterations: vec![
                "The Ming Dynasty continued maritime exploration".to_string(),
                "Chinese colonies established on American west coast".to_string(),
                "European colonization met organized resistance".to_string(),
            ],
        }
    }
}
