//! Narrative dialogue and choice UI

use macroquad::prelude::*;
use crate::rendering::palette;
use crate::systems::narrative::{NarrativeEvent, EventChoice};
use super::Button;

/// Displays narrative events and captures player choices
pub struct DialogueBox {
    /// Current event being displayed
    event: Option<NarrativeEvent>,

    /// Choice buttons
    choice_buttons: Vec<Button>,

    /// Selected choice ID (if any)
    selected_choice: Option<String>,

    /// Animation state
    reveal_progress: f32,

    /// Text revealed so far
    revealed_chars: usize,
}

impl DialogueBox {
    pub fn new() -> Self {
        Self {
            event: None,
            choice_buttons: Vec::new(),
            selected_choice: None,
            reveal_progress: 0.0,
            revealed_chars: 0,
        }
    }

    /// Show a new narrative event
    pub fn show(&mut self, event: NarrativeEvent) {
        // Create buttons for each choice
        let box_width = 600.0;
        let button_height = 40.0;
        let button_spacing = 8.0;
        let start_y = screen_height() / 2.0 + 50.0;

        self.choice_buttons = event
            .choices
            .iter()
            .enumerate()
            .map(|(i, choice)| {
                Button::new(
                    &choice.text,
                    (screen_width() - box_width) / 2.0 + 20.0,
                    start_y + (button_height + button_spacing) * i as f32,
                    box_width - 40.0,
                    button_height,
                )
            })
            .collect();

        self.event = Some(event);
        self.selected_choice = None;
        self.reveal_progress = 0.0;
        self.revealed_chars = 0;
    }

    /// Hide the dialogue box
    pub fn hide(&mut self) {
        self.event = None;
        self.choice_buttons.clear();
        self.selected_choice = None;
    }

    /// Is dialogue currently visible?
    pub fn is_visible(&self) -> bool {
        self.event.is_some()
    }

    /// Get the selected choice (consumes it)
    pub fn take_choice(&mut self) -> Option<String> {
        self.selected_choice.take()
    }

    /// Update dialogue state
    pub fn update(&mut self, dt: f32) {
        if let Some(event) = &self.event {
            // Text reveal animation
            let reveal_speed = 60.0; // chars per second
            self.reveal_progress += dt * reveal_speed;
            self.revealed_chars = (self.reveal_progress as usize).min(event.description.len());

            // Skip reveal on click
            if is_mouse_button_pressed(MouseButton::Left) && self.revealed_chars < event.description.len() {
                self.revealed_chars = event.description.len();
                self.reveal_progress = self.revealed_chars as f32;
            }

            // Only allow choice selection after text is revealed
            if self.revealed_chars >= event.description.len() {
                for (i, button) in self.choice_buttons.iter_mut().enumerate() {
                    if button.update() {
                        if let Some(choice) = event.choices.get(i) {
                            self.selected_choice = Some(choice.id.clone());
                        }
                    }
                }
            }
        }
    }

    /// Render the dialogue box
    pub fn render(&self) {
        if let Some(event) = &self.event {
            let box_width = 600.0;
            let box_height = 400.0;
            let x = (screen_width() - box_width) / 2.0;
            let y = (screen_height() - box_height) / 2.0;

            // Dim background
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));

            // Main box
            draw_rectangle(x, y, box_width, box_height, palette::UI_BACKGROUND);
            draw_rectangle_lines(x, y, box_width, box_height, 2.0, palette::HIGHLIGHT);

            // Title
            let title_dims = measure_text(&event.title, None, 24, 1.0);
            draw_text(
                &event.title,
                x + (box_width - title_dims.width) / 2.0,
                y + 30.0,
                24.0,
                palette::HIGHLIGHT,
            );

            // Separator
            draw_line(x + 20.0, y + 45.0, x + box_width - 20.0, y + 45.0, 1.0, palette::UI_BORDER);

            // Description (with text reveal)
            let revealed_text: String = event.description.chars().take(self.revealed_chars).collect();
            self.draw_wrapped_text(&revealed_text, x + 20.0, y + 60.0, box_width - 40.0, 16.0);

            // Choice buttons (only if text is fully revealed)
            if self.revealed_chars >= event.description.len() {
                for button in &self.choice_buttons {
                    button.render();
                }
            }

            // Continue prompt if still revealing
            if self.revealed_chars < event.description.len() {
                let prompt = "Click to continue...";
                let prompt_dims = measure_text(prompt, None, 14, 1.0);
                let alpha = ((get_time() * 2.0).sin() * 0.5 + 0.5) as f32;
                draw_text(
                    prompt,
                    x + box_width - prompt_dims.width - 20.0,
                    y + box_height - 20.0,
                    14.0,
                    Color::new(1.0, 1.0, 1.0, alpha),
                );
            }
        }
    }

    fn draw_wrapped_text(&self, text: &str, x: f32, y: f32, max_width: f32, font_size: f32) {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut line = String::new();
        let mut line_y = y;
        let line_height = font_size * 1.4;

        for word in words {
            let test_line = if line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", line, word)
            };

            let dims = measure_text(&test_line, None, font_size as u16, 1.0);

            if dims.width > max_width && !line.is_empty() {
                draw_text(&line, x, line_y, font_size, palette::TEXT_PRIMARY);
                line = word.to_string();
                line_y += line_height;
            } else {
                line = test_line;
            }
        }

        if !line.is_empty() {
            draw_text(&line, x, line_y, font_size, palette::TEXT_PRIMARY);
        }
    }
}

impl Default for DialogueBox {
    fn default() -> Self {
        Self::new()
    }
}
