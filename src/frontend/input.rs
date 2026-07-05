//! Selection, orders, and building placement via 3D ray-picking.
//!
//! Left-click / drag selects; right-click issues a context order (move,
//! attack, or assign a citizen to a work site). When a build is armed from
//! the HUD, left-click drops it on the ground under the cursor if the sim
//! allows it there.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::{BuildingKind, Id, Order, UnitKind, MAP_H, MAP_W, TILE};

use super::camera::MainCamera;
use super::scene::{sim_to_world, world_to_sim};
use super::sim::Sim;
use super::AppState;

/// The player's current selection.
#[derive(Resource, Default)]
pub struct Selection {
    pub units: Vec<Id>,
    pub building: Option<Id>,
}

/// A building armed for placement (set by the HUD, consumed on click).
#[derive(Resource, Default)]
pub struct PlacementMode {
    pub kind: Option<BuildingKind>,
}

/// A transient message shown at the bottom of the screen (errors, hints).
#[derive(Resource, Default)]
pub struct Toast {
    pub text: String,
    pub timer: f32,
}

impl Toast {
    pub fn show(&mut self, msg: impl Into<String>) {
        self.text = msg.into();
        self.timer = 2.5;
    }
}

/// Pointer state used to distinguish a click from a drag-select.
#[derive(Resource, Default)]
struct DragState {
    start: Option<Vec2>,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Selection>()
            .init_resource::<PlacementMode>()
            .init_resource::<Toast>()
            .init_resource::<DragState>()
            .add_systems(OnEnter(AppState::Playing), clear_selection)
            .add_systems(
                Update,
                (tick_toast, handle_pointer).run_if(in_state(AppState::Playing)),
            );
    }
}

const PLAYER: usize = Sim::PLAYER;

fn clear_selection(
    mut selection: ResMut<Selection>,
    mut placement: ResMut<PlacementMode>,
) {
    selection.units.clear();
    selection.building = None;
    placement.kind = None;
}

fn tick_toast(time: Res<Time>, mut toast: ResMut<Toast>) {
    if toast.timer > 0.0 {
        toast.timer -= time.delta_secs();
    }
}

/// Ray-cast the cursor onto the ground plane (Y=0) and return the sim XY.
/// Terrain relief is small relative to camera height, so the flat-plane
/// approximation keeps orders responsive without per-triangle picking.
fn cursor_to_sim(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (cam, cam_tf) = camera.single().ok()?;
    let ray = cam.viewport_to_world(cam_tf, cursor).ok()?;
    // Intersect with the ground plane Y=0.
    let denom = ray.direction.y;
    if denom.abs() < 1e-5 {
        return None;
    }
    let t = -ray.origin.y / denom;
    if t < 0.0 {
        return None;
    }
    let hit = ray.origin + ray.direction * t;
    Some(world_to_sim(hit))
}

#[allow(clippy::too_many_arguments)]
fn handle_pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut sim: ResMut<Sim>,
    mut selection: ResMut<Selection>,
    mut placement: ResMut<PlacementMode>,
    mut toast: ResMut<Toast>,
    mut drag: ResMut<DragState>,
) {
    // Cancel placement with Escape or right-click.
    if placement.kind.is_some()
        && (keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right))
    {
        placement.kind = None;
        return;
    }

    let Some(sim_pos) = cursor_to_sim(&windows, &camera) else {
        return;
    };

    // --- Building placement ------------------------------------------------
    if let Some(kind) = placement.kind {
        if mouse.just_pressed(MouseButton::Left) {
            let fp = kind.footprint();
            let tile = (
                (sim_pos.x / TILE).floor() as i32 - fp / 2,
                (sim_pos.y / TILE).floor() as i32 - fp / 2,
            );
            match sim.world.can_place(PLAYER, kind, tile) {
                Ok(()) => {
                    let cost = kind.cost();
                    if sim.world.nations[PLAYER].stockpile.pay(&cost) {
                        sim.world.place_building(PLAYER, kind, tile, false);
                        if !keys.pressed(KeyCode::ShiftLeft) {
                            placement.kind = None;
                        }
                    } else {
                        toast.show(format!("Not enough resources ({})", cost.describe()));
                    }
                }
                Err(e) => toast.show(e),
            }
        }
        return;
    }

    // --- Selection (click or drag box) -------------------------------------
    if mouse.just_pressed(MouseButton::Left) {
        drag.start = window_cursor(&windows);
    }
    if mouse.just_released(MouseButton::Left) {
        if let (Some(start), Some(end)) = (drag.start.take(), window_cursor(&windows)) {
            if start.distance(end) > 6.0 {
                box_select(&mut sim, &camera, &mut selection, start, end);
            } else {
                point_select(&sim, sim_pos, &mut selection);
            }
        }
    }

    // --- Orders ------------------------------------------------------------
    if mouse.just_pressed(MouseButton::Right) && !selection.units.is_empty() {
        issue_order(&mut sim, sim_pos, &selection);
    }
}

fn window_cursor(windows: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    windows.single().ok().and_then(|w| w.cursor_position())
}

fn box_select(
    sim: &mut Sim,
    camera: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    selection: &mut Selection,
    start: Vec2,
    end: Vec2,
) {
    let Ok((cam, cam_tf)) = camera.single() else {
        return;
    };
    let min = start.min(end);
    let max = start.max(end);
    selection.units.clear();
    selection.building = None;
    for u in &sim.world.units {
        if u.nation != PLAYER {
            continue;
        }
        let world = sim_to_world(&sim.world.map, u.pos.x, u.pos.y);
        if let Ok(screen) = cam.world_to_viewport(cam_tf, world) {
            if screen.x >= min.x && screen.x <= max.x && screen.y >= min.y && screen.y <= max.y {
                selection.units.push(u.id);
            }
        }
    }
}

fn point_select(sim: &Sim, at: Vec2, selection: &mut Selection) {
    // Nearest own unit within a generous pick radius.
    let mut best: Option<(Id, f32)> = None;
    for u in &sim.world.units {
        if u.nation != PLAYER {
            continue;
        }
        let d = u.pos.distance(at);
        if d < u.radius() + 16.0 && best.map_or(true, |(_, bd)| d < bd) {
            best = Some((u.id, d));
        }
    }
    if let Some((id, _)) = best {
        selection.units = vec![id];
        selection.building = None;
        return;
    }
    // Otherwise a building footprint.
    for b in &sim.world.buildings {
        let he = b.half_extent();
        if (at.x - b.pos.x).abs() <= he && (at.y - b.pos.y).abs() <= he {
            selection.building = Some(b.id);
            selection.units.clear();
            return;
        }
    }
    selection.units.clear();
    selection.building = None;
}

fn issue_order(sim: &mut Sim, at: Vec2, selection: &Selection) {
    let world = &mut sim.world;

    let enemy_unit = world
        .units
        .iter()
        .filter(|u| u.nation != PLAYER)
        .find(|u| u.pos.distance(at) < u.radius() + 14.0)
        .map(|u| u.id);

    let building = world
        .buildings
        .iter()
        .find(|b| {
            let he = b.half_extent();
            (at.x - b.pos.x).abs() <= he && (at.y - b.pos.y).abs() <= he
        })
        .map(|b| (b.id, b.nation, b.kind.output().is_some()));

    let ids = selection.units.clone();

    if let Some(target) = enemy_unit {
        for id in &ids {
            if let Some(u) = world.units.iter_mut().find(|u| u.id == *id) {
                u.order = if u.kind.is_military() {
                    Order::AttackUnit(target)
                } else {
                    Order::Move { dest: at, aggro: false }
                };
            }
        }
        return;
    }

    if let Some((bid, b_nation, has_slots)) = building {
        if b_nation != PLAYER {
            for id in &ids {
                if let Some(u) = world.units.iter_mut().find(|u| u.id == *id) {
                    if u.kind.is_military() {
                        u.order = Order::AttackBuilding(bid);
                    }
                }
            }
            return;
        }
        if has_slots {
            for id in &ids {
                if let Some(u) = world.units.iter_mut().find(|u| u.id == *id) {
                    u.order = if u.kind == UnitKind::Citizen {
                        Order::Work { building: bid }
                    } else {
                        Order::Move { dest: at, aggro: true }
                    };
                }
            }
            return;
        }
    }

    // Plain move in a loose grid formation.
    for (i, id) in ids.iter().enumerate() {
        let offset = Vec2::new(
            ((i % 5) as f32 - 2.0) * 20.0,
            ((i / 5) as f32) * 20.0,
        );
        if let Some(u) = world.units.iter_mut().find(|u| u.id == *id) {
            let aggro = u.kind.is_military();
            u.order = Order::Move {
                dest: (at + offset).clamp(
                    Vec2::ZERO,
                    Vec2::new(MAP_W as f32 * TILE, MAP_H as f32 * TILE),
                ),
                aggro,
            };
        }
    }
}
