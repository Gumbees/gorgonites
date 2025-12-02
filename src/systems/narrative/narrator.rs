//! The AI Narrator - the heart of Gorgonites
//!
//! This module handles communication with LLMs to generate
//! dynamic narrative content.

use serde::{Deserialize, Serialize};
use crate::game::Era;
use super::{NarrativeEvent, ChoiceHistory, TurningPoint};

/// Configuration for the AI narrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarratorConfig {
    /// API endpoint for LLM
    pub api_endpoint: String,

    /// API key (loaded from env/config)
    #[serde(skip)]
    pub api_key: String,

    /// Model to use
    pub model: String,

    /// Temperature for generation (0.0 - 1.0)
    pub temperature: f32,

    /// Maximum tokens in response
    pub max_tokens: u32,

    /// System prompt defining narrator personality
    pub system_prompt: String,
}

impl Default for NarratorConfig {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            api_key: String::new(),
            model: "claude-sonnet-4-20250514".to_string(),
            temperature: 0.8,
            max_tokens: 1024,
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }
}

const DEFAULT_SYSTEM_PROMPT: &str = r#"
You are the Narrator of Gorgonites, an alternate history strategy game. Your role is to:

1. Generate compelling historical scenarios with meaningful choices
2. React to player decisions with dramatic consequences
3. Track how the timeline diverges from real history
4. Create memorable characters, factions, and crises
5. Force impossible choices - there are no "right" answers

Your tone should be:
- Dramatic but grounded in historical plausibility (at first)
- Increasingly strange as divergence increases
- Never preachy or moralistic - just present the situation
- Occasionally wryly humorous about the chaos

When divergence is low (0-20%), events should feel historically adjacent.
When divergence is high (50%+), embrace the weird. Rome discovers steam power?
China colonizes Europe? Let the timeline go wild.

Always present 2-4 choices. Make them all have clear trade-offs.
Never make one choice obviously "correct".
"#;

/// The narrator instance that generates content
pub struct Narrator {
    config: NarratorConfig,
    /// Context about current game state for coherent generation
    context: NarratorContext,
}

/// Context provided to the AI for generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NarratorContext {
    pub current_era: Era,
    pub divergence_score: f32,
    pub player_faction: String,
    pub choice_summary: String,
    pub active_characters: Vec<String>,
    pub recent_events: Vec<String>,
    pub current_crisis: Option<String>,
}

impl Narrator {
    pub fn new(config: NarratorConfig) -> Self {
        Self {
            config,
            context: NarratorContext::default(),
        }
    }

    /// Update context from game state
    pub fn update_context(
        &mut self,
        era: Era,
        divergence: f32,
        history: &ChoiceHistory,
    ) {
        self.context.current_era = era;
        self.context.divergence_score = divergence;
        self.context.choice_summary = history.generate_summary();
    }

    /// Generate a new narrative event
    pub async fn generate_event(&self) -> Result<NarrativeEvent, NarratorError> {
        // TODO: Implement actual API call
        // For now, return a placeholder
        Ok(self.placeholder_event())
    }

    /// Generate consequences for a choice
    pub async fn generate_consequences(
        &self,
        event: &NarrativeEvent,
        choice_id: &str,
    ) -> Result<ConsequenceNarration, NarratorError> {
        // TODO: Implement actual API call
        let _ = (event, choice_id);
        Ok(ConsequenceNarration {
            immediate_text: "Your choice echoes through history...".to_string(),
            turning_point: None,
            follow_up_event: None,
        })
    }

    fn placeholder_event(&self) -> NarrativeEvent {
        use super::events::{EventChoice, EventEffect};

        NarrativeEvent {
            id: "placeholder_001".to_string(),
            title: "The Crossroads".to_string(),
            description: format!(
                "In the {} era, your civilization faces a pivotal moment...",
                self.context.current_era.display_name()
            ),
            era: self.context.current_era,
            choices: vec![
                EventChoice {
                    id: "choice_a".to_string(),
                    text: "Embrace tradition".to_string(),
                    hint: Some("Stability, but at what cost?".to_string()),
                    effects: vec![EventEffect::ModifyDivergence { amount: -2.0 }],
                    requirements: vec![],
                },
                EventChoice {
                    id: "choice_b".to_string(),
                    text: "Forge a new path".to_string(),
                    hint: Some("Innovation brings chaos...".to_string()),
                    effects: vec![EventEffect::ModifyDivergence { amount: 5.0 }],
                    requirements: vec![],
                },
            ],
            blocking: true,
            priority: 50,
        }
    }
}

/// Result of generating consequences
#[derive(Debug, Clone)]
pub struct ConsequenceNarration {
    /// Immediate narrative response
    pub immediate_text: String,

    /// If this was a turning point
    pub turning_point: Option<TurningPoint>,

    /// Follow-up event to queue
    pub follow_up_event: Option<NarrativeEvent>,
}

/// Errors from narrator operations
#[derive(Debug, thiserror::Error)]
pub enum NarratorError {
    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Rate limited, try again later")]
    RateLimited,

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}
