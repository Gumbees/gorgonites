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
use super::scene::{load_tiled_linear, load_tiled_srgb, sim_to_world, Battlefield, WORLD};
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
    /// glTF character models (KayKit, CC0) per unit line — the real 3D units.
    unit_scene: HashMap<UnitKind, Handle<Scene>>,
    /// Primitive fallback meshes for unit lines without a model (e.g. siege).
    unit_mesh: HashMap<UnitKind, Handle<Mesh>>,
    building_mesh: HashMap<BuildingKind, Handle<Mesh>>,
    military_mat: Vec<Handle<StandardMaterial>>,
    building_mat: HashMap<(BuildingKind, usize), Handle<StandardMaterial>>,
    scaffold_mat: Handle<StandardMaterial>,
    /// Thin nation-coloured "team disc" placed under each unit.
    team_disc_mesh: Handle<Mesh>,
    team_disc_mat: Vec<Handle<StandardMaterial>>,
}

/// Which KayKit character model represents each unit line, and how tall the
/// model stands (Bevy units) so we can scale it to the battlefield.
fn unit_model(kind: UnitKind) -> Option<(&'static str, f32)> {
    match kind {
        UnitKind::Citizen => Some(("models/units/Rogue_Hooded.glb#Scene0", 1.8)),
        UnitKind::Infantry => Some(("models/units/Knight.glb#Scene0", 1.8)),
        UnitKind::Ranged => Some(("models/units/Rogue.glb#Scene0", 1.8)),
        UnitKind::Cavalry => Some(("models/units/Barbarian.glb#Scene0", 1.9)),
        UnitKind::Siege => None, // procedural fallback mesh
    }
}

/// Target on-field height for a unit line (Bevy units).
fn unit_field_height(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Citizen => 2.0,
        UnitKind::Infantry => 2.3,
        UnitKind::Ranged => 2.1,
        UnitKind::Cavalry => 2.7,
        UnitKind::Siege => 1.4,
    }
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
    asset_server: Res<AssetServer>,
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
        let mut mesh = Mesh::from(Cuboid::new(span, building_height(kind), span));
        // Tangents let the wall normal maps catch the light.
        let _ = mesh.generate_tangents();
        building_mesh.insert(kind, meshes.add(mesh));
    }

    // CC0 PBR wall textures (ambientCG), one surface per building family.
    let stone_col = load_tiled_srgb(&asset_server, "textures/stone/color.jpg");
    let stone_nrm = load_tiled_linear(&asset_server, "textures/stone/normal.jpg");
    let wood_col = load_tiled_srgb(&asset_server, "textures/wood/color.jpg");
    let wood_nrm = load_tiled_linear(&asset_server, "textures/wood/normal.jpg");
    let metal_col = load_tiled_srgb(&asset_server, "textures/metal/color.jpg");
    let metal_nrm = load_tiled_linear(&asset_server, "textures/metal/normal.jpg");

    let mut military_mat = Vec::new();
    let mut building_mat = HashMap::new();
    for (i, nation) in sim.world.nations.iter().enumerate() {
        let c = nation.color;
        military_mat.push(materials.add(StandardMaterial {
            base_color: Color::srgb(c[0], c[1], c[2]),
            perceptual_roughness: 0.7,
            metallic: 0.15,
            ..Default::default()
        }));
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
            let (col, nrm, metallic) = match building_surface(kind) {
                Surface::Stone => (stone_col.clone(), stone_nrm.clone(), 0.0),
                Surface::Wood => (wood_col.clone(), wood_nrm.clone(), 0.0),
                Surface::Metal => (metal_col.clone(), metal_nrm.clone(), 0.7),
            };
            // A gentle nation tint over the real texture keeps sides readable.
            let tint = Color::srgb(0.72 + c[0] * 0.4, 0.72 + c[1] * 0.4, 0.72 + c[2] * 0.4);
            building_mat.insert(
                (kind, i),
                materials.add(StandardMaterial {
                    base_color: tint,
                    base_color_texture: Some(col),
                    normal_map_texture: Some(nrm),
                    perceptual_roughness: 0.85,
                    metallic,
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

    // Load the CC0 KayKit character models once.
    let mut unit_scene = HashMap::new();
    for kind in [
        UnitKind::Citizen,
        UnitKind::Infantry,
        UnitKind::Ranged,
        UnitKind::Cavalry,
    ] {
        if let Some((path, _)) = unit_model(kind) {
            unit_scene.insert(kind, asset_server.load(path));
        }
    }

    // A flat disc under each unit carries its nation colour (the models keep
    // their own textures, so ownership is read from the team disc).
    let team_disc_mesh = meshes.add(Cylinder::new(0.7, 0.06));
    let team_disc_mat: Vec<_> = sim
        .world
        .nations
        .iter()
        .map(|n| {
            materials.add(StandardMaterial {
                base_color: Color::srgb(n.color[0], n.color[1], n.color[2]),
                perceptual_roughness: 0.6,
                emissive: LinearRgba::rgb(
                    n.color[0] * 0.2,
                    n.color[1] * 0.2,
                    n.color[2] * 0.2,
                ),
                ..Default::default()
            })
        })
        .collect();

    commands.insert_resource(Assets3d {
        unit_scene,
        unit_mesh,
        building_mesh,
        military_mat,
        building_mat,
        scaffold_mat,
        team_disc_mesh,
        team_disc_mat,
    });
}

/// Which real material surfaces a building family.
enum Surface {
    Stone,
    Wood,
    Metal,
}

fn building_surface(kind: BuildingKind) -> Surface {
    match kind {
        BuildingKind::City | BuildingKind::University | BuildingKind::Mine => Surface::Stone,
        BuildingKind::OilWell => Surface::Metal,
        _ => Surface::Wood,
    }
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

    // KayKit characters face +Z in their own space; rotate so they turn to
    // face their sim heading.
    const MODEL_YAW: f32 = FRAC_PI_2;

    for u in &w.units {
        seen.insert(u.id, ());
        let ground = sim_to_world(&w.map, u.pos.x, u.pos.y);
        let model = unit_model(u.kind).is_some();
        // Character models sit on their feet at ground level; primitive
        // fallbacks are centred, so lift them by half their height.
        let pos = if model {
            ground
        } else {
            ground + Vec3::Y * unit_field_height(u.kind) * 0.5
        };
        let rot = Quat::from_rotation_y(-u.facing + MODEL_YAW);

        match index.units.get(&u.id) {
            Some(&e) => {
                if let Ok(mut t) = transforms.get_mut(e) {
                    t.translation = pos;
                    t.rotation = rot;
                }
            }
            None => {
                let e = if let Some((_, native_h)) = unit_model(u.kind) {
                    let scale = unit_field_height(u.kind) / native_h;
                    commands
                        .spawn((
                            Battlefield,
                            SceneRoot(assets.unit_scene[&u.kind].clone()),
                            Transform::from_translation(pos)
                                .with_rotation(rot)
                                .with_scale(Vec3::splat(scale)),
                        ))
                        .with_children(|p| {
                            // Counter-scale the disc so it stays a fixed size.
                            p.spawn((
                                Mesh3d(assets.team_disc_mesh.clone()),
                                MeshMaterial3d(assets.team_disc_mat[u.nation].clone()),
                                Transform::from_translation(Vec3::Y * 0.02 / scale)
                                    .with_scale(Vec3::splat(1.0 / scale)),
                            ));
                        })
                        .id()
                } else {
                    // Siege and any other model-less line: primitive fallback.
                    commands
                        .spawn((
                            Battlefield,
                            Mesh3d(assets.unit_mesh[&u.kind].clone()),
                            MeshMaterial3d(assets.military_mat[u.nation].clone()),
                            Transform::from_translation(pos).with_rotation(rot),
                        ))
                        .id()
                };
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

/// Target on-field height for a unit line (Bevy units) — used for the
/// primitive fallback offset and health-bar placement.
fn unit_visual_height(kind: UnitKind) -> f32 {
    unit_field_height(kind)
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
                        Battlefield,
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
