//! Core game state and loop management

use macroquad::prelude::*;

mod state;
mod era;

pub use state::*;
pub use era::*;

use crate::audio::AudioManager;
use crate::config::GameConfig;
use crate::sprites::{SpriteGenerator, SpriteManager, RenderedSprite, FogSystem};
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

    /// Audio manager for music and sound
    audio: Option<AudioManager>,

    /// Previous game state (for detecting transitions)
    prev_state: GameState,

    /// AI sprite generator
    sprite_generator: Option<SpriteGenerator>,

    /// Rendered sprites on screen
    sprite_manager: SpriteManager,

    /// Last mouse position for tracking movement
    last_mouse_pos: (f32, f32),

    /// Cooldown timer for sprite generation
    sprite_cooldown: f32,

    /// 8-bit fog particle system
    fog_system: FogSystem,
}

impl Game {
    pub fn new() -> Self {
        // Load config
        let config = GameConfig::load("config.ini");

        // Initialize audio (may fail on systems without audio)
        let mut audio = match AudioManager::new() {
            Ok(manager) => Some(manager),
            Err(e) => {
                tracing::warn!("Audio initialization failed: {}. Continuing without audio.", e);
                None
            }
        };

        // Start menu music immediately
        if let Some(ref mut audio_manager) = audio {
            audio_manager.play_menu_music();
        }

        // Initialize sprite generator (may fail if Ollama not available)
        let sprite_generator = if config.ollama.enabled {
            Some(SpriteGenerator::new(&config.ollama))
        } else {
            tracing::info!("Ollama disabled in config, sprite generation will use fallbacks");
            None
        };

        Self {
            state: GameState::MainMenu,
            current_era: Era::StoneAge,
            divergence_score: 0.0,
            show_debug: false,
            world: World::new(),
            main_menu: MainMenu::new(),
            menu_needs_layout: true,
            audio,
            prev_state: GameState::MainMenu,
            sprite_generator,
            sprite_manager: SpriteManager::new(50), // Max 50 sprites on screen
            last_mouse_pos: (0.0, 0.0),
            sprite_cooldown: 0.0,
            fog_system: FogSystem::new(),
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
        // Handle state transitions for audio
        self.handle_audio_transitions();

        // Fog disabled - too many draw calls causing driver issues
        // self.fog_system.update(dt);

        // Update sprite generation (runs in all states)
        self.update_sprites(dt);

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
                        self.sprite_manager.clear(); // Clear menu sprites
                        tracing::info!("Starting new game!");
                    }
                    MenuAction::Continue => {
                        // TODO: Load save game
                        self.state = GameState::Playing;
                        self.sprite_manager.clear();
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

    /// Update sprite generation based on mouse movement
    fn update_sprites(&mut self, dt: f32) {
        // Update generator state (recheck availability periodically)
        if let Some(ref mut generator) = self.sprite_generator {
            generator.update();
        }

        // Decrease cooldown
        self.sprite_cooldown = (self.sprite_cooldown - dt).max(0.0);

        // Get current mouse position
        let (mx, my) = mouse_position();

        // Check if mouse moved significantly
        let (lx, ly) = self.last_mouse_pos;
        let dist = ((mx - lx).powi(2) + (my - ly).powi(2)).sqrt();

        // Generate sprite on significant mouse movement (and not on cooldown)
        // Only if generator is available and not busy
        if dist > 50.0 && self.sprite_cooldown <= 0.0 {
            if let Some(ref mut generator) = self.sprite_generator {
                // Only request if available and not already generating
                if generator.request_sprite(mx, my) {
                    self.last_mouse_pos = (mx, my);
                    self.sprite_cooldown = 0.3; // Short cooldown - actual wait is for generation to complete
                    tracing::debug!("Requested sprite at ({}, {})", mx, my);
                }
            }
        }

        // Poll for completed sprites
        if let Some(ref mut generator) = self.sprite_generator {
            if let Some((desc, x, y)) = generator.poll() {
                tracing::info!("Generated sprite: {} at ({}, {})", desc.name, x, y);
                self.sprite_manager.add(RenderedSprite::new(desc, x, y));
            }
        }
    }

    /// Handle audio based on game state transitions
    fn handle_audio_transitions(&mut self) {
        if self.state == self.prev_state {
            return;
        }

        // State changed - handle audio transitions
        if let Some(ref mut audio) = self.audio {
            match (self.prev_state, self.state) {
                // Entering main menu - start music
                (_, GameState::MainMenu) => {
                    audio.play_menu_music();
                }
                // Leaving main menu - stop music
                (GameState::MainMenu, _) => {
                    audio.stop_music();
                }
                _ => {}
            }
        }

        self.prev_state = self.state;
    }

    /// Initialize audio (call after entering main menu for first time)
    pub fn start_menu_music(&mut self) {
        if let Some(ref mut audio) = self.audio {
            audio.play_menu_music();
        }
    }

    /// Render the current frame
    pub fn render(&self) {
        // Fog disabled - too many draw calls causing driver issues
        // self.fog_system.render();

        match self.state {
            GameState::MainMenu => {
                // Render sprites BEHIND the menu
                self.sprite_manager.render();
                // Then render menu on top
                self.main_menu.render();
                // Render status info on top of everything
                self.render_sprite_info();
            }
            GameState::Playing => {
                self.world.render();
                self.sprite_manager.render();
                self.render_hud();
            }
            GameState::Paused => {
                self.world.render();
                self.sprite_manager.render();
                self.render_pause_overlay();
            }
            _ => {}
        }
    }

    /// Render sprite generation info and Ollama status
    fn render_sprite_info(&self) {
        let count = self.sprite_manager.count();

        match &self.sprite_generator {
            Some(gen) => {
                if !gen.is_available() {
                    // Ollama not available - show warning
                    let warning = if gen.is_checking() {
                        "Connecting to Ollama..."
                    } else {
                        "Ollama not available - waiting for connection"
                    };
                    draw_text(warning, 10.0, 20.0, 16.0, Color::from_rgba(255, 200, 100, 230));
                } else if gen.is_busy() {
                    // Currently generating
                    let info = format!("Sprites: {} | Generating...", count);
                    draw_text(&info, 10.0, 20.0, 16.0, Color::from_rgba(100, 255, 100, 200));
                } else {
                    // Ready to generate
                    let info = format!("Sprites: {} | Move mouse to generate", count);
                    draw_text(&info, 10.0, 20.0, 16.0, Color::from_rgba(200, 200, 200, 200));
                }
            }
            None => {
                // No generator configured
                draw_text("Ollama disabled in config", 10.0, 20.0, 16.0, Color::from_rgba(150, 150, 150, 150));
            }
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
