//! Core game state and loop management

use macroquad::prelude::*;

mod state;
mod era;

pub use state::*;
pub use era::*;

use crate::ui::{MainMenu, MenuAction};

/// The main game struct holding all state
pub struct Game {
    /// Current game state (menu, playing, paused, etc.)
    pub state: GameState,

    /// Current historical era
    pub current_era: Era,

    /// How far the timeline has diverged from reality (0.0 - 100.0)
    pub divergence_score: f32,

    /// Show debug overlay
    pub show_debug: bool,

    /// Game world (entities, map, etc.)
    world: World,

    /// Main menu UI
    main_menu: MainMenu,

    /// Track if menu needs layout update
    menu_needs_layout: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::MainMenu,
            current_era: Era::StoneAge,
            divergence_score: 0.0,
            show_debug: false,
            world: World::new(),
            main_menu: MainMenu::new(),
            menu_needs_layout: true,
        }
    }

    /// Handle all input for the current frame
    pub fn handle_input(&mut self) {
        // Toggle debug with F3
        if is_key_pressed(KeyCode::F3) {
            self.show_debug = !self.show_debug;
        }

        // Escape to pause/unpause or return to menu
        if is_key_pressed(KeyCode::Escape) {
            match self.state {
                GameState::Playing => self.state = GameState::Paused,
                GameState::Paused => self.state = GameState::Playing,
                GameState::MainMenu => {},
                _ => {},
            }
        }
    }

    /// Update game logic
    pub fn update(&mut self, dt: f32) {
        match self.state {
            GameState::MainMenu => {
                // Layout menu if needed (e.g., on resize)
                if self.menu_needs_layout {
                    self.main_menu.layout(false); // TODO: check for save game
                    self.menu_needs_layout = false;
                }

                // Update menu and handle actions
                match self.main_menu.update(dt) {
                    MenuAction::NewGame => {
                        self.state = GameState::Playing;
                        tracing::info!("Starting new game!");
                    }
                    MenuAction::Continue => {
                        // TODO: Load save game
                        self.state = GameState::Playing;
                        tracing::info!("Continuing game...");
                    }
                    MenuAction::Settings => {
                        // TODO: Open settings menu
                        tracing::info!("Settings not yet implemented");
                    }
                    MenuAction::Quit => {
                        tracing::info!("Quitting game...");
                        std::process::exit(0);
                    }
                    MenuAction::None => {}
                }
            }
            GameState::Playing => {
                self.world.update(dt);
            }
            GameState::Paused => {
                // Paused - no updates
            }
            _ => {}
        }
    }

    /// Render the current frame
    pub fn render(&self) {
        match self.state {
            GameState::MainMenu => {
                self.main_menu.render();
            }
            GameState::Playing => {
                self.world.render();
                self.render_hud();
            }
            GameState::Paused => {
                self.world.render();
                self.render_pause_overlay();
            }
            _ => {}
        }
    }

    fn render_hud(&self) {
        // Era indicator
        let era_text = format!("{:?}", self.current_era);
        draw_text(&era_text, 10.0, screen_height() - 30.0, 20.0, WHITE);

        // Divergence meter
        let div_text = format!("Divergence: {:.1}%", self.divergence_score);
        draw_text(&div_text, 10.0, screen_height() - 10.0, 16.0, YELLOW);
    }

    fn render_pause_overlay(&self) {
        // Dim overlay
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        // Pause text
        let text = "PAUSED";
        let size = 48.0;
        let dims = measure_text(text, None, size as u16, 1.0);
        draw_text(
            text,
            (screen_width() - dims.width) / 2.0,
            screen_height() / 2.0,
            size,
            WHITE,
        );

        let hint = "Press ESC to resume";
        let hint_size = 20.0;
        let hint_dims = measure_text(hint, None, hint_size as u16, 1.0);
        draw_text(
            hint,
            (screen_width() - hint_dims.width) / 2.0,
            screen_height() / 2.0 + 40.0,
            hint_size,
            GRAY,
        );
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

/// The game world containing all entities and state
pub struct World {
    // TODO: Add ECS world, map, etc.
}

impl World {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, _dt: f32) {
        // TODO: Run game systems
    }

    pub fn render(&self) {
        // TODO: Render world

        // Placeholder: draw a simple grid to show something is happening
        let grid_color = Color::from_rgba(40, 40, 50, 255);
        let cell_size = 32.0;

        for x in (0..(screen_width() as i32)).step_by(cell_size as usize) {
            draw_line(x as f32, 0.0, x as f32, screen_height(), 1.0, grid_color);
        }
        for y in (0..(screen_height() as i32)).step_by(cell_size as usize) {
            draw_line(0.0, y as f32, screen_width(), y as f32, 1.0, grid_color);
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
