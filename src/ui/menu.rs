//! Main menu system

use macroquad::prelude::*;
use super::Button;
use crate::rendering::{palette, draw_text_shadowed};

/// Menu action returned when a button is clicked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    None,
    NewGame,
    Continue,
    Settings,
    Quit,
}

/// The main menu screen
pub struct MainMenu {
    buttons: Vec<(Button, MenuAction)>,
    title_pulse: f32,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            title_pulse: 0.0,
        }
    }

    /// Initialize/update button positions based on screen size
    pub fn layout(&mut self, has_save: bool) {
        self.buttons.clear();

        let screen_w = screen_width();
        let screen_h = screen_height();

        let button_width = 200.0;
        let button_height = 45.0;
        let button_spacing = 15.0;
        let start_y = screen_h / 2.0;

        let mut y = start_y;
        let x = (screen_w - button_width) / 2.0;

        // Continue button (only if save exists)
        if has_save {
            self.buttons.push((
                Button::new("Continue", x, y, button_width, button_height),
                MenuAction::Continue,
            ));
            y += button_height + button_spacing;
        }

        // New Game
        self.buttons.push((
            Button::new("New Game", x, y, button_width, button_height),
            MenuAction::NewGame,
        ));
        y += button_height + button_spacing;

        // Settings
        self.buttons.push((
            Button::new("Settings", x, y, button_width, button_height),
            MenuAction::Settings,
        ));
        y += button_height + button_spacing;

        // Quit
        self.buttons.push((
            Button::new("Quit", x, y, button_width, button_height),
            MenuAction::Quit,
        ));
    }

    /// Update menu state and return any action
    pub fn update(&mut self, dt: f32) -> MenuAction {
        self.title_pulse += dt * 2.0;

        for (button, action) in &mut self.buttons {
            if button.update() {
                return *action;
            }
        }

        MenuAction::None
    }

    /// Render the menu
    pub fn render(&self) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Background gradient effect
        for i in 0..20 {
            let y = i as f32 * (screen_h / 20.0);
            let alpha = 0.02 + (i as f32 * 0.005);
            draw_rectangle(
                0.0,
                y,
                screen_w,
                screen_h / 20.0,
                Color::new(0.1, 0.1, 0.15, alpha),
            );
        }

        // Title with glow effect
        let title = "GORGONITES";
        let title_size = 72.0;
        let title_dims = measure_text(title, None, title_size as u16, 1.0);
        let title_x = (screen_w - title_dims.width) / 2.0;
        let title_y = screen_h / 3.5;

        // Pulsing glow
        let glow_alpha = (self.title_pulse.sin() * 0.3 + 0.5) as f32;
        let glow_color = Color::new(0.4, 0.6, 1.0, glow_alpha * 0.3);

        // Draw glow layers
        for offset in [4.0, 2.0] {
            draw_text(title, title_x - offset, title_y, title_size, glow_color);
            draw_text(title, title_x + offset, title_y, title_size, glow_color);
            draw_text(title, title_x, title_y - offset, title_size, glow_color);
            draw_text(title, title_x, title_y + offset, title_size, glow_color);
        }

        // Main title
        draw_text_shadowed(title, title_x, title_y, title_size, WHITE);

        // Subtitle
        let subtitle = "An AI-Driven Alternate History";
        let sub_size = 22.0;
        let sub_dims = measure_text(subtitle, None, sub_size as u16, 1.0);
        draw_text(
            subtitle,
            (screen_w - sub_dims.width) / 2.0,
            title_y + 45.0,
            sub_size,
            palette::TEXT_SECONDARY,
        );

        // Render buttons
        for (button, _) in &self.buttons {
            button.render();
        }

        // Version info
        let version = "v0.1.0";
        let version_dims = measure_text(version, None, 14, 1.0);
        draw_text(
            version,
            screen_w - version_dims.width - 10.0,
            screen_h - 10.0,
            14.0,
            palette::TEXT_SECONDARY,
        );
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}
