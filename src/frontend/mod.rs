//! Bevy 3D frontend.
//!
//! The battlefield simulation (`crate::game`) is engine-agnostic; this module
//! is everything Bevy: the 3D scene (terrain mesh, sun + shadows, fog),
//! entity syncing, the RTS camera, picking/orders, and the HUD.
//!
//! Art direction stays Company of Heroes: muted earth tones, low warm sun,
//! atmospheric distance fog, tracers and smoke — now with real lighting.

pub mod camera;
pub mod capture;
pub mod hud;
pub mod input;
pub mod scene;
pub mod sim;
pub mod sync;

use bevy::prelude::*;

/// Top-level application flow.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Gorgonites".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.055, 0.065, 0.075)))
        .init_state::<AppState>()
        .add_plugins((
            sim::SimPlugin,
            scene::ScenePlugin,
            camera::CameraPlugin,
            input::InputPlugin,
            sync::SyncPlugin,
            hud::HudPlugin,
            capture::CapturePlugin,
        ))
        .run();
}
