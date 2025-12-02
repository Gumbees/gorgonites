//! Core game state and loop management

use macroquad::prelude::*;

mod state;
mod era;

pub use state::*;
pub use era::*;

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
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::MainMenu,
            current_era: Era::StoneAge,
            divergence_score: 0.0,
            show_debug: false,
            world: World::new(),
        }
    }

    /// Handle all input for the current frame
    pub fn handle_input(&mut self) {
        // Toggle debug with F3
        if is_key_pressed(KeyCode::F3) {
            self.show_debug = !self.show_debug;
        }

        // Escape to pause/unpause or exit menu
        if is_key_pressed(KeyCode::Escape) {
            match self.state {
                GameState::Playing => self.state = GameState::Paused,
                GameState::Paused => self.state = GameState::Playing,
                GameState::MainMenu => {}, // Could open quit confirm
                _ => {},
            }
        }

        // Temporary: Space to start game from menu
        if is_key_pressed(KeyCode::Space) && self.state == GameState::MainMenu {
            self.state = GameState::Playing;
            tracing::info!("Game started!");
        }
    }

    /// Update game logic
    pub fn update(&mut self, dt: f32) {
        match self.state {
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
                self.render_main_menu();
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

    fn render_main_menu(&self) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Title
        let title = "GORGONITES";
        let title_size = 64.0;
        let title_dims = measure_text(title, None, title_size as u16, 1.0);
        draw_text(
            title,
            (screen_w - title_dims.width) / 2.0,
            screen_h / 3.0,
            title_size,
            WHITE,
        );

        // Subtitle
        let subtitle = "An AI-Driven Alternate History";
        let sub_size = 24.0;
        let sub_dims = measure_text(subtitle, None, sub_size as u16, 1.0);
        draw_text(
            subtitle,
            (screen_w - sub_dims.width) / 2.0,
            screen_h / 3.0 + 40.0,
            sub_size,
            GRAY,
        );

        // Start prompt
        let prompt = "Press SPACE to begin";
        let prompt_size = 20.0;
        let prompt_dims = measure_text(prompt, None, prompt_size as u16, 1.0);

        // Pulsing alpha effect
        let alpha = ((get_time() * 2.0).sin() * 0.5 + 0.5) as f32;
        draw_text(
            prompt,
            (screen_w - prompt_dims.width) / 2.0,
            screen_h * 2.0 / 3.0,
            prompt_size,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
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
