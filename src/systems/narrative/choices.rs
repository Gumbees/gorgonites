//! Choice tracking and consequence management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks all choices made during a game session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChoiceHistory {
    /// All choices made, keyed by event ID
    choices: HashMap<String, ChoiceRecord>,

    /// Major turning points that significantly altered the timeline
    turning_points: Vec<TurningPoint>,
}

impl ChoiceHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a choice
    pub fn record(&mut self, event_id: String, choice_id: String, divergence_impact: f32) {
        self.choices.insert(
            event_id.clone(),
            ChoiceRecord {
                event_id,
                choice_id,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                divergence_impact,
            },
        );
    }

    /// Add a turning point
    pub fn add_turning_point(&mut self, point: TurningPoint) {
        self.turning_points.push(point);
    }

    /// Get all choices
    pub fn all_choices(&self) -> &HashMap<String, ChoiceRecord> {
        &self.choices
    }

    /// Get turning points
    pub fn turning_points(&self) -> &[TurningPoint] {
        &self.turning_points
    }

    /// Generate a summary for AI context
    pub fn generate_summary(&self) -> String {
        let mut summary = String::from("Timeline choices:\n");

        for point in &self.turning_points {
            summary.push_str(&format!(
                "- {}: {} (Divergence: +{:.1}%)\n",
                point.title, point.description, point.divergence_caused
            ));
        }

        summary
    }
}

/// Record of a single choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceRecord {
    pub event_id: String,
    pub choice_id: String,
    pub timestamp: u64,
    pub divergence_impact: f32,
}

/// A major turning point in the timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurningPoint {
    /// Title of this turning point
    pub title: String,

    /// What happened
    pub description: String,

    /// What would have happened in our timeline
    pub original_history: String,

    /// How much this diverged the timeline
    pub divergence_caused: f32,

    /// Game turn/year when this occurred
    pub when: u32,
}

/// The butterfly effect - how early choices compound
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ButterflyTracker {
    /// Multiplier for how much new choices affect divergence
    /// Increases as timeline diverges more
    pub chaos_multiplier: f32,

    /// Count of cascading effects from choices
    pub cascade_count: u32,
}

impl ButterflyTracker {
    pub fn new() -> Self {
        Self {
            chaos_multiplier: 1.0,
            cascade_count: 0,
        }
    }

    /// Calculate actual divergence impact considering butterfly effect
    pub fn calculate_impact(&self, base_impact: f32, current_divergence: f32) -> f32 {
        // Higher divergence = more chaotic, bigger swings
        let chaos_factor = 1.0 + (current_divergence / 100.0) * self.chaos_multiplier;
        base_impact * chaos_factor
    }

    /// A cascade occurred (choice triggered secondary effects)
    pub fn record_cascade(&mut self) {
        self.cascade_count += 1;
        self.chaos_multiplier += 0.1;
    }
}
