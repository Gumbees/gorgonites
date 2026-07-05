//! Mirror the simulation into the Bevy scene each frame.
//!
//! Units and buildings become persistent PBR meshes tracked by sim id (spawn
//! on appear, move each frame, despawn on death). Transient and grid-shaped
//! visuals — territory borders, selection rings, health bars, particles — are
//! drawn with immediate-mode gizmos.

use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::game::{BuildingKind, Id, ParticleKind, UnitKind, MAP_H, MAP_W, TILE};

use super::camera::MainCamera;
use super::input::Selection;
use super::scene::{sim_to_world, SceneRoot, WORLD};
use super::sim::Sim;
use super::AppState;

/// Maps sim ids to their Bevy entities.
#[derive(Resource, Default)]
pub struct EntityIndex {
    units: HashMap<Id, Entity>,
    buildings: HashMap<Id, Entity>,
}

/// Prebuilt meshes/materials so per-frame sync never touches the asset store.
#[derive(Resource)]
pub struct Assets3d {
    unit_mesh: HashMap<UnitKind, Handle<Mesh>>,
    building_mesh: HashMap<BuildingKind, Handle<Mesh>>,
    citizen_mat: Vec<Handle<StandardMaterial>>,
    military_mat: Vec<Handle<StandardMaterial>>,
    building_mat: HashMap<(BuildingKind, usize), Handle<StandardMaterial>>,
    scaffold_mat: Handle<StandardMaterial>,
}

pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityIndex>()
            .add_systems(OnEnter(AppState::Playing), build_assets)
            .add_systems(OnExit(AppState::Playing), clear_index)
            .add_systems(
                Update,
                (sync_units, sync_buildings, draw_overlays)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn clear_index(mut index: ResMut<EntityIndex>) {
    index.units.clear();
    index.buildings.clear();
}

/// Building footprint side in Bevy units.
fn building_span(kind: BuildingKind) -> f32 {
    kind.footprint() as f32 * TILE * WORLD
}

fn building_height(kind: BuildingKind) -> f32 {
    match kind {
        BuildingKind::City => 6.5,
        BuildingKind::Barracks => 4.0,
        BuildingKind::University => 4.5,
        BuildingKind::Market => 3.2,
        BuildingKind::Mine | BuildingKind::LumberCamp => 2.6,
        BuildingKind::OilWell => 5.0,
        BuildingKind::Farm => 0.8,
    }
}

fn build_assets(
    mut commands: Commands,
    sim: Res<Sim>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut unit_mesh = HashMap::new();
    unit_mesh.insert(
        UnitKind::Citizen,
        meshes.add(Capsule3d::new(0.5, 0.9)),
    );
    unit_mesh.insert(UnitKind::Infantry, meshes.add(Capsule3d::new(0.55, 1.2)));
    unit_mesh.insert(UnitKind::Ranged, meshes.add(Capsule3d::new(0.5, 1.15)));
    unit_mesh.insert(UnitKind::Cavalry, meshes.add(Capsule3d::new(0.7, 1.6)));
    unit_mesh.insert(
        UnitKind::Siege,
        meshes.add(Cuboid::new(2.2, 1.4, 3.0)),
    );

    let mut building_mesh = HashMap::new();
    for kind in [
        BuildingKind::City,
        BuildingKind::Farm,
        BuildingKind::LumberCamp,
        BuildingKind::Mine,
        BuildingKind::Market,
        BuildingKind::University,
        BuildingKind::Barracks,
        BuildingKind::OilWell,
    ] {
        let span = building_span(kind) * 0.86;
        building_mesh.insert(kind, meshes.add(Cuboid::new(span, building_height(kind), span)));
    }

    let mut citizen_mat = Vec::new();
    let mut military_mat = Vec::new();
    let mut building_mat = HashMap::new();
    for (i, nation) in sim.world.nations.iter().enumerate() {
        let c = nation.color;
        citizen_mat.push(materials.add(StandardMaterial {
            base_color: Color::srgb(c[0] * 0.7 + 0.25, c[1] * 0.7 + 0.22, c[2] * 0.7 + 0.16),
            perceptual_roughness: 0.85,
            ..Default::default()
        }));
        military_mat.push(materials.add(StandardMaterial {
            base_color: Color::srgb(c[0], c[1], c[2]),
            perceptual_roughness: 0.7,
            metallic: 0.15,
            ..Default::default()
        }));
        for (kind, base) in building_base_colors() {
            // Blend the kind's material colour with a touch of nation tint.
            let col = Color::srgb(
                base[0] * 0.78 + c[0] * 0.22,
                base[1] * 0.78 + c[1] * 0.22,
                base[2] * 0.78 + c[2] * 0.22,
            );
            building_mat.insert(
                (kind, i),
                materials.add(StandardMaterial {
                    base_color: col,
                    perceptual_roughness: 0.9,
                    ..Default::default()
                }),
            );
        }
    }

    let scaffold_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.5, 0.4, 0.55),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        ..Default::default()
    });

    commands.insert_resource(Assets3d {
        unit_mesh,
        building_mesh,
        citizen_mat,
        military_mat,
        building_mat,
        scaffold_mat,
    });
}

fn building_base_colors() -> [(BuildingKind, [f32; 3]); 8] {
    [
        (BuildingKind::City, [0.52, 0.49, 0.44]),
        (BuildingKind::Farm, [0.55, 0.47, 0.28]),
        (BuildingKind::LumberCamp, [0.40, 0.32, 0.22]),
        (BuildingKind::Mine, [0.44, 0.43, 0.41]),
        (BuildingKind::Market, [0.56, 0.44, 0.30]),
        (BuildingKind::University, [0.52, 0.53, 0.58]),
        (BuildingKind::Barracks, [0.42, 0.38, 0.32]),
        (BuildingKind::OilWell, [0.24, 0.23, 0.22]),
    ]
}

fn sync_units(
    mut commands: Commands,
    sim: Res<Sim>,
    assets: Res<Assets3d>,
    mut index: ResMut<EntityIndex>,
    mut transforms: Query<&mut Transform>,
) {
    let w = &sim.world;
    let mut seen: HashMap<Id, ()> = HashMap::new();

    for u in &w.units {
        seen.insert(u.id, ());
        let ground = sim_to_world(&w.map, u.pos.x, u.pos.y);
        let height = unit_visual_height(u.kind);
        let pos = ground + Vec3::Y * height * 0.5;
        let rot = Quat::from_rotation_y(-u.facing + FRAC_PI_2);

        match index.units.get(&u.id) {
            Some(&e) => {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.translation = pos;
                    t.rotation = rot;
                }
            }
            None => {
                let mat = if u.kind == UnitKind::Citizen {
                    assets.citizen_mat[u.nation].clone()
                } else {
                    assets.military_mat[u.nation].clone()
                };
                let e = commands
                    .spawn((
                        SceneRoot,
                        Mesh3d(assets.unit_mesh[&u.kind].clone()),
                        MeshMaterial3d(mat),
                        Transform::from_translation(pos).with_rotation(rot),
                    ))
                    .id();
                index.units.insert(u.id, e);
            }
        }
    }

    // Despawn units that died this frame.
    index.units.retain(|id, e| {
        if seen.contains_key(id) {
            true
        } else {
            commands.entity(*e).despawn();
            false
        }
    });
}

fn unit_visual_height(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Citizen => 1.9,
        UnitKind::Infantry => 2.3,
        UnitKind::Ranged => 2.15,
        UnitKind::Cavalry => 3.0,
        UnitKind::Siege => 1.4,
    }
}

fn sync_buildings(
    mut commands: Commands,
    sim: Res<Sim>,
    assets: Res<Assets3d>,
    mut index: ResMut<EntityIndex>,
    mut q: Query<(&mut Transform, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let w = &sim.world;
    let mut seen: HashMap<Id, ()> = HashMap::new();

    for b in &w.buildings {
        seen.insert(b.id, ());
        let ground = sim_to_world(&w.map, b.pos.x, b.pos.y);
        let h = building_height(b.kind);
        // Under construction: rise from the ground as it completes.
        let grow = 0.15 + 0.85 * b.construction;
        let pos = ground + Vec3::Y * h * grow * 0.5;
        let scale = Vec3::new(1.0, grow, 1.0);
        let want_mat = if b.is_complete() {
            assets.building_mat[&(b.kind, b.nation)].clone()
        } else {
            assets.scaffold_mat.clone()
        };

        match index.buildings.get(&b.id) {
            Some(&e) => {
                if let Ok((mut t, mut mat)) = q.get_mut(e) {
                    t.translation = pos;
                    t.scale = scale;
                    if mat.0.id() != want_mat.id() {
                        mat.0 = want_mat;
                    }
                }
            }
            None => {
                let e = commands
                    .spawn((
                        SceneRoot,
                        Mesh3d(assets.building_mesh[&b.kind].clone()),
                        MeshMaterial3d(want_mat),
                        Transform::from_translation(pos).with_scale(scale),
                    ))
                    .id();
                index.buildings.insert(b.id, e);
            }
        }
    }

    index.buildings.retain(|id, e| {
        if seen.contains_key(id) {
            true
        } else {
            commands.entity(*e).despawn();
            false
        }
    });
}

/// Territory borders, selection markers, health bars, and particles.
fn draw_overlays(
    sim: Res<Sim>,
    selection: Res<Selection>,
    camera: Query<&Transform, With<MainCamera>>,
    mut gizmos: Gizmos,
) {
    let w = &sim.world;

    draw_territory(w, &mut gizmos);

    // Camera-right vector for billboarded health bars.
    let cam_right = camera
        .single()
        .map(|t| t.right().as_vec3())
        .unwrap_or(Vec3::X);

    // Health bars for damaged units.
    for u in &w.units {
        let frac = (u.hp / u.stats.max_hp).clamp(0.0, 1.0);
        if frac < 0.999 {
            let base = sim_to_world(&w.map, u.pos.x, u.pos.y)
                + Vec3::Y * (unit_visual_height(u.kind) + 0.6);
            health_bar(&mut gizmos, base, cam_right, frac);
        }
    }

    // Selection rings.
    for id in &selection.units {
        if let Some(u) = w.unit(*id) {
            let c = sim_to_world(&w.map, u.pos.x, u.pos.y) + Vec3::Y * 0.15;
            let r = (u.radius() * WORLD).max(0.6) + 0.4;
            gizmos.circle(
                Isometry3d::new(c, Quat::from_rotation_x(FRAC_PI_2)),
                r,
                Color::srgb(0.85, 0.95, 0.8),
            );
        }
    }
    if let Some(id) = selection.building {
        if let Some(b) = w.building(id) {
            let c = sim_to_world(&w.map, b.pos.x, b.pos.y) + Vec3::Y * 0.15;
            let r = building_span(b.kind) * 0.75;
            gizmos.circle(
                Isometry3d::new(c, Quat::from_rotation_x(FRAC_PI_2)),
                r,
                Color::srgb(0.85, 0.95, 0.8),
            );
        }
    }

    draw_particles(w, &mut gizmos);
}

fn health_bar(gizmos: &mut Gizmos, center: Vec3, right: Vec3, frac: f32) {
    let half = 1.1;
    let left = center - right * half;
    let full_right = center + right * half;
    let fill = left + right * (half * 2.0 * frac);
    let color = if frac > 0.6 {
        Color::srgb(0.4, 0.75, 0.35)
    } else if frac > 0.3 {
        Color::srgb(0.85, 0.7, 0.25)
    } else {
        Color::srgb(0.8, 0.3, 0.25)
    };
    gizmos.line(left, full_right, Color::srgb(0.1, 0.1, 0.1));
    gizmos.line(left, fill, color);
}

/// Draw a national border line wherever ownership changes between adjacent
/// tiles — the Rise of Nations territory read, projected onto the terrain.
fn draw_territory(w: &Sim3World, gizmos: &mut Gizmos) {
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let Some(owner) = w.tile_owner(x, y) else {
                continue;
            };
            let nc = w.nations[owner as usize].color;
            let color = Color::srgb(
                (nc[0] * 1.3).min(1.0),
                (nc[1] * 1.3).min(1.0),
                (nc[2] * 1.3).min(1.0),
            );
            // Draw right/bottom edges when the neighbour differs; interior
            // left/top edges are covered by that neighbour's own right/bottom
            // pass, so only the map-boundary left/top edges are added here.
            if w.tile_owner(x + 1, y) != Some(owner) {
                edge_line(w, gizmos, x + 1, y, x + 1, y + 1, color);
            }
            if w.tile_owner(x, y + 1) != Some(owner) {
                edge_line(w, gizmos, x, y + 1, x + 1, y + 1, color);
            }
            if x == 0 {
                edge_line(w, gizmos, x, y, x, y + 1, color);
            }
            if y == 0 {
                edge_line(w, gizmos, x, y, x + 1, y, color);
            }
        }
    }
}

type Sim3World = crate::game::World;

fn edge_line(w: &Sim3World, gizmos: &mut Gizmos, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    let a = tile_corner(w, x0, y0);
    let b = tile_corner(w, x1, y1);
    gizmos.line(a, b, color);
}

fn tile_corner(w: &Sim3World, gx: i32, gy: i32) -> Vec3 {
    let sx = gx as f32 * TILE;
    let sy = gy as f32 * TILE;
    Vec3::new(sx * WORLD, (w.map.elevation_world(sx, sy) + 0.4) * WORLD + 0.15, sy * WORLD)
}

fn draw_particles(w: &Sim3World, gizmos: &mut Gizmos) {
    for p in &w.particles {
        let ground = w.map.elevation_world(p.pos.x, p.pos.y);
        let base = Vec3::new(p.pos.x * WORLD, ground * WORLD, p.pos.y * WORLD);
        let frac = (p.life / p.max_life).clamp(0.0, 1.0);
        let col = Color::srgba(p.color[0], p.color[1], p.color[2], p.color[3] * frac);
        match p.kind {
            ParticleKind::Tracer { to } => {
                let end = Vec3::new(
                    to.x * WORLD,
                    (w.map.elevation_world(to.x, to.y) + 12.0) * WORLD,
                    to.y * WORLD,
                );
                let start = base + Vec3::Y * 8.0 * WORLD;
                gizmos.line(start, end, col);
            }
            ParticleKind::Flash => {
                gizmos.sphere(
                    Isometry3d::from_translation(base + Vec3::Y * 8.0 * WORLD),
                    p.size * WORLD * 1.5,
                    col,
                );
            }
            ParticleKind::Smoke => {
                gizmos.sphere(
                    Isometry3d::from_translation(base + Vec3::Y * (6.0 + p.size) * WORLD),
                    p.size * WORLD,
                    col,
                );
            }
            ParticleKind::Blood | ParticleKind::Spark => {
                gizmos.sphere(
                    Isometry3d::from_translation(base + Vec3::Y * 5.0 * WORLD),
                    (p.size * WORLD).max(0.1),
                    col,
                );
            }
        }
    }
}
