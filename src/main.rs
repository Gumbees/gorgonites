use macroquad::prelude::*;

mod game;
mod ecs;
mod systems;
mod ai;
mod rendering;
mod ui;
mod assets;
mod audio;
mod config;
mod sprites;

use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Gorgonites".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Gorgonites starting...");

    // Initialize game state
    let mut game = Game::new();

    // Main game loop
    loop {
        // Calculate delta time
        let dt = get_frame_time();

        // Handle input
        game.handle_input();

        // Update game state
        game.update(dt);

        // Render
        clear_background(Color::from_rgba(20, 20, 30, 255));
        game.render();

        // Debug info (toggle with F3)
        if game.show_debug {
            render_debug_info(&game);
        }

        next_frame().await
    }
}

fn render_debug_info(game: &Game) {
    let fps = get_fps();
    let debug_text = format!(
        "FPS: {} | Era: {:?} | Divergence: {:.1}%",
        fps,
        game.current_era,
        game.divergence_score
    );

    draw_text(&debug_text, 10.0, 20.0, 16.0, WHITE);
}
