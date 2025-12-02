//! Heads-up display elements

use macroquad::prelude::*;
use crate::rendering::{palette, draw_progress_bar};
use crate::game::Era;
use crate::systems::rts::ResourceStockpile;

/// Main game HUD
pub struct Hud {
    /// Show minimap
    pub show_minimap: bool,

    /// Show resource bar
    pub show_resources: bool,

    /// Current era for display
    pub era: Era,

    /// Current divergence
    pub divergence: f32,

    /// Player resources
    pub resources: ResourceStockpile,

    /// Current year
    pub year: i32,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            show_minimap: true,
            show_resources: true,
            era: Era::StoneAge,
            divergence: 0.0,
            resources: ResourceStockpile::default(),
            year: -10000,
        }
    }
}

impl Hud {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update HUD state from game state
    pub fn update(&mut self, era: Era, divergence: f32, resources: &ResourceStockpile, year: i32) {
        self.era = era;
        self.divergence = divergence;
        self.resources = resources.clone();
        self.year = year;
    }

    /// Render the HUD
    pub fn render(&self) {
        self.render_top_bar();
        self.render_resource_bar();

        if self.show_minimap {
            self.render_minimap();
        }

        self.render_divergence_meter();
    }

    fn render_top_bar(&self) {
        let bar_height = 30.0;

        // Background
        draw_rectangle(0.0, 0.0, screen_width(), bar_height, palette::UI_BACKGROUND);
        draw_line(0.0, bar_height, screen_width(), bar_height, 1.0, palette::UI_BORDER);

        // Era
        draw_text(self.era.display_name(), 10.0, 20.0, 18.0, palette::TEXT_PRIMARY);

        // Year
        let year_str = if self.year < 0 {
            format!("{} BCE", -self.year)
        } else {
            format!("{} CE", self.year)
        };
        let year_dims = measure_text(&year_str, None, 18, 1.0);
        draw_text(
            &year_str,
            screen_width() - year_dims.width - 10.0,
            20.0,
            18.0,
            palette::TEXT_PRIMARY,
        );
    }

    fn render_resource_bar(&self) {
        if !self.show_resources {
            return;
        }

        let bar_y = 35.0;
        let icon_size = 20.0;
        let spacing = 80.0;
        let mut x = 10.0;

        let resources = [
            ("Food", self.resources.food, palette::SUCCESS),
            ("Wood", self.resources.wood, Color::from_rgba(139, 90, 43, 255)),
            ("Stone", self.resources.stone, Color::from_rgba(128, 128, 128, 255)),
            ("Metal", self.resources.metal, Color::from_rgba(192, 192, 192, 255)),
            ("Gold", self.resources.gold, palette::WARNING),
        ];

        for (name, amount, color) in resources {
            // Icon (colored square for now)
            draw_rectangle(x, bar_y, icon_size, icon_size, color);
            draw_rectangle_lines(x, bar_y, icon_size, icon_size, 1.0, palette::UI_BORDER);

            // Amount
            let text = format!("{}", amount);
            draw_text(&text, x + icon_size + 4.0, bar_y + 15.0, 14.0, palette::TEXT_PRIMARY);

            x += spacing;
        }
    }

    fn render_minimap(&self) {
        let size = 150.0;
        let margin = 10.0;
        let x = screen_width() - size - margin;
        let y = screen_height() - size - margin;

        // Background
        draw_rectangle(x, y, size, size, Color::new(0.1, 0.1, 0.15, 0.9));
        draw_rectangle_lines(x, y, size, size, 2.0, palette::UI_BORDER);

        // Placeholder content
        draw_text("Minimap", x + 5.0, y + 15.0, 12.0, palette::TEXT_SECONDARY);

        // TODO: Actually render minimap
    }

    fn render_divergence_meter(&self) {
        let width = 200.0;
        let height = 20.0;
        let margin = 10.0;
        let x = screen_width() - width - margin;
        let y = margin + 35.0;

        // Label
        draw_text("Divergence", x, y - 5.0, 14.0, palette::TEXT_SECONDARY);

        // Progress bar
        let progress = self.divergence / 100.0;
        let color = if self.divergence < 20.0 {
            palette::SUCCESS
        } else if self.divergence < 50.0 {
            palette::WARNING
        } else {
            palette::DANGER
        };

        draw_progress_bar(x, y, width, height, progress, color);

        // Percentage
        let pct_text = format!("{:.1}%", self.divergence);
        let pct_dims = measure_text(&pct_text, None, 14, 1.0);
        draw_text(
            &pct_text,
            x + (width - pct_dims.width) / 2.0,
            y + height / 2.0 + 5.0,
            14.0,
            palette::TEXT_PRIMARY,
        );
    }
}

/// Selection info panel (shows selected units/buildings)
pub struct SelectionPanel {
    /// Currently selected entity count
    pub selected_count: usize,

    /// Selected entity type name
    pub selected_type: Option<String>,

    /// Health of selected (average if multiple)
    pub health: Option<(f32, f32)>,
}

impl Default for SelectionPanel {
    fn default() -> Self {
        Self {
            selected_count: 0,
            selected_type: None,
            health: None,
        }
    }
}

impl SelectionPanel {
    pub fn render(&self) {
        if self.selected_count == 0 {
            return;
        }

        let panel_width = 200.0;
        let panel_height = 80.0;
        let x = 10.0;
        let y = screen_height() - panel_height - 10.0;

        // Background
        draw_rectangle(x, y, panel_width, panel_height, palette::UI_BACKGROUND);
        draw_rectangle_lines(x, y, panel_width, panel_height, 2.0, palette::UI_BORDER);

        // Type and count
        let type_name = self.selected_type.as_deref().unwrap_or("Unknown");
        let header = if self.selected_count > 1 {
            format!("{} x{}", type_name, self.selected_count)
        } else {
            type_name.to_string()
        };
        draw_text(&header, x + 10.0, y + 20.0, 16.0, palette::TEXT_PRIMARY);

        // Health bar
        if let Some((current, max)) = self.health {
            draw_text("HP:", x + 10.0, y + 45.0, 12.0, palette::TEXT_SECONDARY);
            let health_pct = current / max;
            let health_color = if health_pct > 0.6 {
                palette::SUCCESS
            } else if health_pct > 0.3 {
                palette::WARNING
            } else {
                palette::DANGER
            };
            draw_progress_bar(x + 35.0, y + 35.0, panel_width - 50.0, 15.0, health_pct, health_color);
        }
    }
}
