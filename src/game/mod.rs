//! Core game state and loop management.
//!
//! Gameplay is Rise of Nations: national borders, attrition, commerce-capped
//! economy, age advancement, city capture, capital-loss countdown. The
//! narrative/divergence layer from the original vision rides on top.

use macroquad::prelude::*;

mod ai_nation;
mod entities;
mod era;
mod interface;
mod mapgen;
mod render_world;
mod state;
mod world;

pub use entities::*;
pub use era::*;
pub use interface::Interface;
pub use mapgen::{GameMap, Terrain, MAP_H, MAP_W, TILE};
pub use state::*;
pub use world::{age_up_cost, Nation, World};

use crate::rendering::GameCamera;

/// The main game struct holding all state.
pub struct Game {
    pub state: GameState,
    /// Player nation's era (mirrors `world.nations[0].age`).
    pub current_era: Era,
    /// How far the timeline has diverged from reality (0.0 - 100.0).
    pub divergence_score: f32,
    pub show_debug: bool,
    pub world: World,
    pub camera: GameCamera,
    pub interface: Interface,
    /// Set once the battle ends: did the player win?
    victory: bool,
}

impl Game {
    pub fn new() -> Self {
        let world = World::new();
        let mut camera = GameCamera::new();
        if let Some(capital) = world.building(world.nations[0].capital) {
            camera.position = capital.pos;
        }
        Self {
            state: GameState::MainMenu,
            current_era: Era::StoneAge,
            divergence_score: 0.0,
            show_debug: false,
            world,
            camera,
            interface: Interface::new(),
            victory: false,
        }
    }

    fn restart(&mut self) {
        self.world = World::new();
        self.interface = Interface::new();
        self.victory = false;
        self.current_era = Era::StoneAge;
        if let Some(capital) = self.world.building(self.world.nations[0].capital) {
            self.camera.position = capital.pos;
        }
        self.camera.zoom = 1.0;
    }

    /// Handle all input for the current frame.
    pub fn handle_input(&mut self, dt: f32) {
        if is_key_pressed(KeyCode::F3) {
            self.show_debug = !self.show_debug;
        }

        match self.state {
            GameState::MainMenu => {
                if is_key_pressed(KeyCode::Space) {
                    self.restart();
                    self.state = GameState::Playing;
                    tracing::info!("Battle started");
                }
            }
            GameState::Playing => {
                if is_key_pressed(KeyCode::Escape) && self.interface.placing.is_none() {
                    self.state = GameState::Paused;
                    return;
                }
                self.camera.handle_input(dt);
                if let Some(target) = self.interface.minimap_world_target() {
                    self.camera.position = target;
                }
                self.clamp_camera();
                self.interface
                    .handle_input(&mut self.world, &self.camera, dt);
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Space) {
                    self.state = GameState::MainMenu;
                }
            }
            _ => {}
        }
    }

    fn clamp_camera(&mut self) {
        let max = vec2(MAP_W as f32 * TILE, MAP_H as f32 * TILE);
        self.camera.position = self.camera.position.clamp(Vec2::ZERO, max);
    }

    /// Update game logic.
    pub fn update(&mut self, dt: f32) {
        if self.state != GameState::Playing {
            return;
        }
        self.world.update(dt);
        self.current_era = Era::from_index(self.world.nations[0].age);
        // Every age climbed past history's pace nudges the timeline.
        self.divergence_score =
            (self.world.nations[0].age as f32 * 6.0 + self.world.game_time / 120.0).min(100.0);

        if let Some(winner) = self.world.winner {
            self.victory = winner == 0;
            self.state = GameState::GameOver;
        } else if self.world.nations[0].defeated {
            self.victory = false;
            self.state = GameState::GameOver;
        }
    }

    /// Render the current frame.
    pub fn render(&mut self, dt: f32) {
        match self.state {
            GameState::MainMenu => self.render_main_menu(),
            GameState::Playing => {
                render_world::render_world(&self.world, &self.camera);
                self.interface.draw(&mut self.world, &self.camera, dt);
            }
            GameState::Paused => {
                render_world::render_world(&self.world, &self.camera);
                self.render_pause_overlay();
            }
            GameState::GameOver => {
                render_world::render_world(&self.world, &self.camera);
                self.render_game_over();
            }
            _ => {}
        }
    }

    fn render_main_menu(&self) {
        let screen_w = screen_width();
        let screen_h = screen_height();

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

        let subtitle = "Borders. Attrition. Eight ages of war.";
        let sub_size = 24.0;
        let sub_dims = measure_text(subtitle, None, sub_size as u16, 1.0);
        draw_text(
            subtitle,
            (screen_w - sub_dims.width) / 2.0,
            screen_h / 3.0 + 40.0,
            sub_size,
            GRAY,
        );

        let lines = [
            "Left-drag: select   Right-click: move / attack / assign workers",
            "Citizens staff farms, camps, mines, markets, universities",
            "Build only inside your borders. Enemy soil bleeds your troops.",
            "Capture the enemy capital to win — and hold your own.",
        ];
        for (i, line) in lines.iter().enumerate() {
            let dims = measure_text(line, None, 16, 1.0);
            draw_text(
                line,
                (screen_w - dims.width) / 2.0,
                screen_h / 2.0 + i as f32 * 22.0,
                16.0,
                Color::new(0.65, 0.65, 0.6, 1.0),
            );
        }

        let prompt = "Press SPACE to begin";
        let prompt_dims = measure_text(prompt, None, 20, 1.0);
        let alpha = ((get_time() * 2.0).sin() * 0.5 + 0.5) as f32;
        draw_text(
            prompt,
            (screen_w - prompt_dims.width) / 2.0,
            screen_h * 3.0 / 4.0,
            20.0,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }

    fn render_pause_overlay(&self) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        let text = "PAUSED";
        let dims = measure_text(text, None, 48, 1.0);
        draw_text(
            text,
            (screen_width() - dims.width) / 2.0,
            screen_height() / 2.0,
            48.0,
            WHITE,
        );
        let hint = "Press ESC to resume";
        let hint_dims = measure_text(hint, None, 20, 1.0);
        draw_text(
            hint,
            (screen_width() - hint_dims.width) / 2.0,
            screen_height() / 2.0 + 40.0,
            20.0,
            GRAY,
        );
    }

    fn render_game_over(&self) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.75),
        );
        let (text, color) = if self.victory {
            ("VICTORY", Color::new(0.7, 0.9, 0.55, 1.0))
        } else {
            ("DEFEAT", Color::new(0.9, 0.3, 0.25, 1.0))
        };
        let dims = measure_text(text, None, 64, 1.0);
        draw_text(
            text,
            (screen_width() - dims.width) / 2.0,
            screen_height() / 2.0 - 20.0,
            64.0,
            color,
        );
        let summary = format!(
            "Reached {} — {} kills in {:.0} minutes",
            Era::from_index(self.world.nations[0].age).display_name(),
            self.world.nations[0].kills,
            self.world.game_time / 60.0
        );
        let sdims = measure_text(&summary, None, 20, 1.0);
        draw_text(
            &summary,
            (screen_width() - sdims.width) / 2.0,
            screen_height() / 2.0 + 20.0,
            20.0,
            Color::new(0.8, 0.8, 0.75, 1.0),
        );
        let hint = "Press SPACE for the main menu";
        let hint_dims = measure_text(hint, None, 20, 1.0);
        draw_text(
            hint,
            (screen_width() - hint_dims.width) / 2.0,
            screen_height() / 2.0 + 56.0,
            20.0,
            GRAY,
        );
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
