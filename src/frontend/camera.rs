//! RTS camera: an angled overhead view that pans (edge/keys/minimap), zooms
//! (scroll → dolly along the view direction), and rotates (Q/E).
//!
//! The camera is a yaw pivot at a ground focus point; the actual `Camera3d`
//! sits back and above it, giving the raised, slightly-tilted Company of
//! Heroes framing rather than a flat top-down.

use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;

use crate::game::{MAP_H, MAP_W, TILE};

use super::scene::{HORIZON, WORLD};
use super::AppState;

#[derive(Resource)]
pub struct RtsCamera {
    /// Ground focus point in Bevy world space (Y ignored for panning).
    pub focus: Vec3,
    pub yaw: f32,
    /// Camera distance from the focus.
    pub distance: f32,
    /// Downward pitch in radians.
    pub pitch: f32,
}

impl Default for RtsCamera {
    fn default() -> Self {
        let cx = MAP_W as f32 * TILE * WORLD * 0.5;
        let cz = MAP_H as f32 * TILE * WORLD * 0.5;
        Self {
            focus: Vec3::new(cx, 0.0, cz),
            yaw: 0.0,
            distance: 90.0,
            pitch: 1.0, // ~57°, a raised oblique
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RtsCamera>()
            .add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(AppState::Playing), recenter_on_capital)
            .add_systems(
                Update,
                control_camera.run_if(in_state(AppState::Playing)),
            )
            .add_systems(PostUpdate, apply_camera);
    }
}

fn spawn_camera(mut commands: Commands, rts: Res<RtsCamera>) {
    let mut cam = Transform::default();
    place(&mut cam, &rts);
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        // HDR is required for bloom to have highlights to bleed.
        Camera {
            hdr: true,
            ..Default::default()
        },
        // Filmic, LUT-free tonemapping (the default TonyMcMapFace needs the
        // tonemapping_luts feature we omit for a lean, headless-friendly build).
        Tonemapping::AcesFitted,
        // Soft glare on the sun disc and bright highlights.
        Bloom {
            intensity: 0.18,
            ..Bloom::NATURAL
        },
        // Atmospheric depth: distant terrain fades into the horizon haze.
        DistanceFog {
            color: HORIZON,
            falloff: FogFalloff::Linear {
                start: 90.0,
                end: 360.0,
            },
            ..Default::default()
        },
        cam,
    ));
}

fn recenter_on_capital(sim: Res<super::sim::Sim>, mut rts: ResMut<RtsCamera>) {
    let w = &sim.world;
    if let Some(cap) = w.building(w.nations[super::sim::Sim::PLAYER].capital) {
        rts.focus = super::scene::sim_to_world(&w.map, cap.pos.x, cap.pos.y);
        rts.focus.y = 0.0;
    }
}

fn control_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll: EventReader<bevy::input::mouse::MouseWheel>,
    mut rts: ResMut<RtsCamera>,
) {
    let dt = time.delta_secs();
    let pan_speed = 60.0 * (rts.distance / 90.0);

    // Movement is relative to yaw so "up" is always screen-up.
    let (sin, cos) = rts.yaw.sin_cos();
    let forward = Vec3::new(-sin, 0.0, -cos);
    let right = Vec3::new(cos, 0.0, -sin);

    let mut mv = Vec3::ZERO;
    if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        mv += forward;
    }
    if keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        mv -= forward;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        mv += right;
    }
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        mv -= right;
    }
    if mv != Vec3::ZERO {
        rts.focus += mv.normalize() * pan_speed * dt;
    }

    if keys.pressed(KeyCode::KeyQ) {
        rts.yaw -= 1.2 * dt;
    }
    if keys.pressed(KeyCode::KeyE) {
        rts.yaw += 1.2 * dt;
    }

    for ev in scroll.read() {
        rts.distance = (rts.distance - ev.y * 8.0).clamp(28.0, 200.0);
    }

    // Keep the focus over the map.
    let max_x = MAP_W as f32 * TILE * WORLD;
    let max_z = MAP_H as f32 * TILE * WORLD;
    rts.focus.x = rts.focus.x.clamp(0.0, max_x);
    rts.focus.z = rts.focus.z.clamp(0.0, max_z);
}

/// Position the camera transform behind/above the focus per yaw/pitch/distance.
fn place(cam: &mut Transform, rts: &RtsCamera) {
    let (sy, cy) = rts.yaw.sin_cos();
    let horiz = rts.pitch.cos() * rts.distance;
    let height = rts.pitch.sin() * rts.distance;
    let offset = Vec3::new(sy * horiz, height, cy * horiz);
    cam.translation = rts.focus + offset;
    cam.look_at(rts.focus, Vec3::Y);
}

fn apply_camera(rts: Res<RtsCamera>, mut q: Query<&mut Transform, With<MainCamera>>) {
    if let Ok(mut t) = q.single_mut() {
        place(&mut t, &rts);
    }
}
