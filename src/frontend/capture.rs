//! Dev/CI capture aid: when `GORG_SCREENSHOT=<path>` is set, jump straight
//! into a battle, let it run a few seconds so the economy and armies populate,
//! save a PNG of the framebuffer, then exit. Lets a headless environment prove
//! the 3D renderer actually produces frames. No effect without the env var.
//!
//! Optional knobs (all env vars):
//! - `GORG_SCREENSHOT_AT`     — seconds before the first shot (default `6.0`).
//! - `GORG_SCREENSHOT_BURST`  — take N shots instead of one, numbered
//!   `<stem>_00.<ext>`, `<stem>_01.<ext>`, … A burst a few hundred ms apart
//!   proves the characters are actually animating, not frozen at bind pose.
//! - `GORG_SCREENSHOT_EVERY`  — seconds between burst shots (default `0.5`).
//! - `GORG_MARCH`             — at this many seconds, order every player unit to
//!   walk a long way east. Lets a headless run exercise the walk animation and
//!   verify unit facing without a human issuing move orders.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use crate::game::Order;

use super::sim::Sim;
use super::AppState;

#[derive(Resource)]
struct Capture {
    path: String,
    elapsed: f32,
    /// When to take the first shot.
    first_at: f32,
    /// Gap between shots in a burst.
    every: f32,
    /// How many shots to take in total.
    total: u32,
    /// How many shots taken so far.
    taken: u32,
    /// If set, second at which to order player units to march (walk test).
    march_at: Option<f32>,
}

pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        let Ok(path) = std::env::var("GORG_SCREENSHOT") else {
            return;
        };
        let first_at = env_f32("GORG_SCREENSHOT_AT", 6.0).max(0.0);
        let every = env_f32("GORG_SCREENSHOT_EVERY", 0.5).max(0.05);
        let total = env_u32("GORG_SCREENSHOT_BURST", 1).max(1);
        let march_at = std::env::var("GORG_MARCH").ok().and_then(|s| s.parse().ok());
        app.insert_resource(Capture {
            path,
            elapsed: 0.0,
            first_at,
            every,
            total,
            taken: 0,
            march_at,
        })
        .add_systems(Startup, jump_into_game)
        .add_systems(Update, (drive_capture, march_units));
    }
}

/// Dev aid: at `GORG_MARCH=<secs>`, order every player unit to walk far east so
/// the walk clip and unit facing can be eyeballed in a headless capture.
fn march_units(
    time: Res<Time>,
    cap: Res<Capture>,
    mut sim: ResMut<Sim>,
    mut timer: Local<f32>,
    mut done: Local<bool>,
) {
    let Some(at) = cap.march_at else { return };
    *timer += time.delta_secs();
    if *done || *timer < at {
        return;
    }
    *done = true;
    let mut n = 0;
    for u in &mut sim.world.units {
        if u.nation == 0 {
            u.order = Order::Move {
                dest: bevy::math::Vec2::new(u.pos.x + 1500.0, u.pos.y),
                aggro: false,
            };
            n += 1;
        }
    }
    info!("capture: ordered {n} player units to march east");
}

fn env_f32(key: &str, default: f32) -> f32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
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

    // Give the scene time to build and units to move before each shot.
    let next_shot_at = cap.first_at + cap.every * cap.taken as f32;
    if cap.taken < cap.total && cap.elapsed > next_shot_at {
        let path = if cap.total == 1 {
            cap.path.clone()
        } else {
            indexed_path(&cap.path, cap.taken)
        };
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
        info!("capture: shot {}/{} -> {}", cap.taken + 1, cap.total, path);
        cap.taken += 1;
    }

    // Hold briefly after the final shot so the async save flushes, then quit.
    let last_shot_at = cap.first_at + cap.every * cap.total.saturating_sub(1) as f32;
    if cap.taken >= cap.total && cap.elapsed > last_shot_at + 2.0 {
        exit.write(AppExit::Success);
    }
}

/// Insert a zero-padded index before the file extension: `shot.png` -> `shot_03.png`.
fn indexed_path(base: &str, i: u32) -> String {
    match base.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}_{i:02}.{ext}"),
        None => format!("{base}_{i:02}"),
    }
}
