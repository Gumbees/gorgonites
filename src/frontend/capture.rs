//! Dev/CI capture aid: when `GORG_SCREENSHOT=<path>` is set, jump straight
//! into a battle, let it run a few seconds so the economy and armies populate,
//! save a PNG of the framebuffer, then exit. Lets a headless environment prove
//! the 3D renderer actually produces frames. No effect without the env var.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use super::AppState;

#[derive(Resource)]
struct Capture {
    path: String,
    elapsed: f32,
    shot: bool,
}

pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        let Ok(path) = std::env::var("GORG_SCREENSHOT") else {
            return;
        };
        app.insert_resource(Capture {
            path,
            elapsed: 0.0,
            shot: false,
        })
        .add_systems(Startup, jump_into_game)
        .add_systems(Update, drive_capture);
    }
}

fn jump_into_game(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Playing);
}

fn drive_capture(
    time: Res<Time>,
    mut cap: ResMut<Capture>,
    mut commands: Commands,
    mut exit: EventWriter<AppExit>,
) {
    cap.elapsed += time.delta_secs();
    // Give the scene time to build and units to move before the shot.
    if !cap.shot && cap.elapsed > 6.0 {
        cap.shot = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(cap.path.clone()));
        info!("capture: screenshot requested -> {}", cap.path);
    }
    // Hold briefly so the async save flushes, then quit.
    if cap.shot && cap.elapsed > 8.0 {
        exit.write(AppExit::Success);
    }
}
