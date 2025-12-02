//! Prompt templates for AI generation

use crate::game::Era;
use crate::systems::timeline::DivergenceCategory;

/// Build the system prompt for the narrator
pub fn narrator_system_prompt(divergence: DivergenceCategory) -> String {
    let tone = match divergence {
        DivergenceCategory::Familiar => "grounded in real history with minor variations",
        DivergenceCategory::Altered => "recognizable but with significant differences",
        DivergenceCategory::Radical => "strange and unfamiliar, embrace the weird",
        DivergenceCategory::Alien => "completely unrecognizable, anything goes",
    };

    format!(
        r#"You are the Narrator of Gorgonites, an alternate history strategy game.

Your narrative style should be {}.

Core responsibilities:
1. Generate historical scenarios with 2-4 meaningful choices
2. React to player decisions with dramatic consequences
3. Create memorable characters and crises
4. Never be preachy - present situations, let players decide

Output format:
Return JSON with this structure:
{{
  "title": "Event title",
  "description": "The narrative description...",
  "choices": [
    {{
      "id": "choice_a",
      "text": "Choice text",
      "hint": "Optional hint about consequences"
    }}
  ]
}}

Be concise but evocative. Focus on human drama and difficult decisions."#,
        tone
    )
}

/// Build a prompt for generating a scenario event
pub fn scenario_prompt(era: Era, context: &str) -> String {
    format!(
        r#"Generate a narrative event for the {} era.

Current context:
{}

Create a scenario that:
- Fits the era's technology and culture
- Presents a genuine dilemma with no clear "right" answer
- Has consequences that will ripple through history
- Involves characters the player might care about

Remember: Every choice should matter. No filler events."#,
        era.display_name(),
        context
    )
}

/// Build a prompt for generating consequences
pub fn consequence_prompt(
    event_title: &str,
    choice_made: &str,
    era: Era,
    divergence: f32,
) -> String {
    format!(
        r#"The player faced "{}" and chose: "{}"

Era: {}
Current divergence: {:.1}%

Describe the immediate and long-term consequences of this choice.
Consider:
- How does this change the immediate situation?
- What new opportunities or threats does this create?
- How might this compound with previous choices?
- What historical events might now never happen (or happen differently)?

Output format:
{{
  "immediate": "What happens right now...",
  "long_term": "In the years to come...",
  "divergence_impact": 5.0,
  "follow_up_event": null or {{ event object }}
}}"#,
        event_title, choice_made, era.display_name(), divergence
    )
}

/// Build a prompt for generating a character
pub fn character_prompt(era: Era, role: &str, faction: &str) -> String {
    format!(
        r#"Create a character for the {} era.

Role: {}
Faction: {}

Create a memorable character with:
- A distinctive name appropriate to the era/culture
- A clear motivation and personality
- A potential for both heroism and villainy
- Relationships that create drama

Output format:
{{
  "name": "Character name",
  "title": "Their role/title",
  "description": "Brief description",
  "motivation": "What drives them",
  "flaw": "A weakness or blind spot",
  "potential_arcs": ["possible story directions"]
}}"#,
        era.display_name(),
        role,
        faction
    )
}

/// Template for historical "what if" scenarios
pub fn what_if_prompt(historical_event: &str, alteration: &str) -> String {
    format!(
        r#"Historical event: {}
What if: {}

Extrapolate the consequences of this change:
1. Immediate effects (first generation)
2. Medium-term effects (1-3 generations)
3. Long-term effects (centuries)
4. Unexpected second-order effects

Be creative but maintain internal consistency."#,
        historical_event, alteration
    )
}
