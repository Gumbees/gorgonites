//! The 3D scene: terrain mesh, water plane, sun, sky, and fog.
//!
//! Coordinate bridge — the sim is 2D `(x, y)` in "sim units" (TILE = 32 per
//! tile). We map that to Bevy's right-handed Y-up space as
//! `(x, elevation, y)`: the sim's Y axis becomes world Z, and terrain relief
//! becomes world Y. `WORLD` scales sim units down so the whole map is a
//! sensible number of Bevy units across.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::game::{GameMap, Terrain, World as SimWorld, MAP_H, MAP_W, TILE};

use super::sim::Sim;
use super::AppState;

/// Sim-unit → Bevy-unit scale. Map is MAP_W*TILE = 3072 sim units wide;
/// at 0.08 that's ~246 Bevy units — a comfortable RTS field.
pub const WORLD: f32 = 0.08;

/// Convert a sim ground position to a 3D point on the terrain surface.
pub fn sim_to_world(map: &GameMap, x: f32, y: f32) -> Vec3 {
    Vec3::new(
        x * WORLD,
        map.elevation_world(x, y) * WORLD,
        y * WORLD,
    )
}

/// Convert a flat 3D point (ignoring height) back to sim XY.
pub fn world_to_sim(p: Vec3) -> Vec2 {
    Vec2::new(p.x / WORLD, p.z / WORLD)
}

/// Marker for the whole battlefield scene so we can tear it down on restart.
#[derive(Component)]
pub struct SceneRoot;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), setup_scene)
            .add_systems(OnExit(AppState::Playing), teardown_scene);
    }
}

fn teardown_scene(mut commands: Commands, roots: Query<Entity, With<SceneRoot>>) {
    for e in &roots {
        commands.entity(e).despawn();
    }
}

fn setup_scene(
    mut commands: Commands,
    sim: Res<Sim>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // (map elevation is sampled inside build_terrain_mesh)

    // --- Sun: a low, warm key light with long shadows (CoH morning) --------
    commands.spawn((
        SceneRoot,
        DirectionalLight {
            color: Color::srgb(1.0, 0.94, 0.82),
            illuminance: 11_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(120.0, 180.0, 60.0)
            .looking_at(Vec3::new(120.0, 0.0, 120.0), Vec3::Y),
    ));

    // Cool ambient fill so shadows aren't black.
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 0.68, 0.82),
        brightness: 380.0,
        ..Default::default()
    });

    // --- Terrain mesh ------------------------------------------------------
    let terrain_mesh = meshes.add(build_terrain_mesh(&sim.world));
    let terrain_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    });
    commands.spawn((
        SceneRoot,
        Mesh3d(terrain_mesh),
        MeshMaterial3d(terrain_mat),
        Transform::default(),
    ));

    // --- Water plane at sea level (semi-transparent) -----------------------
    let map_w = MAP_W as f32 * TILE * WORLD;
    let map_h = MAP_H as f32 * TILE * WORLD;
    let water = meshes.add(Plane3d::default().mesh().size(map_w * 2.0, map_h * 2.0));
    let water_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.16, 0.3, 0.42, 0.82),
        perceptual_roughness: 0.15,
        metallic: 0.1,
        reflectance: 0.5,
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });
    commands.spawn((
        SceneRoot,
        Mesh3d(water),
        MeshMaterial3d(water_mat),
        Transform::from_xyz(map_w / 2.0, -1.6 * WORLD, map_h / 2.0),
    ));
}

/// Build a single terrain mesh with per-vertex colour by terrain class and
/// smooth normals, so the PBR sun shades ridges and valleys.
fn build_terrain_mesh(world: &SimWorld) -> Mesh {
    let map = &world.map;
    let w = MAP_W as usize;
    let h = MAP_H as usize;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((w + 1) * (h + 1));
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity((w + 1) * (h + 1));

    // One vertex per tile corner (grid is (w+1) x (h+1) vertices).
    for gy in 0..=h {
        for gx in 0..=w {
            let sx = gx as f32 * TILE;
            let sy = gy as f32 * TILE;
            let elev = map.elevation_world(sx, sy);
            positions.push([sx * WORLD, elev * WORLD, sy * WORLD]);
            colors.push(terrain_vertex_color(map, gx as i32, gy as i32));
        }
    }

    let mut indices: Vec<u32> = Vec::with_capacity(w * h * 6);
    let stride = (w + 1) as u32;
    for gy in 0..h as u32 {
        for gx in 0..w as u32 {
            let tl = gy * stride + gx;
            let tr = tl + 1;
            let bl = (gy + 1) * stride + gx;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();
    mesh
}

/// Average the terrain colour of the up-to-four tiles meeting at a corner so
/// class boundaries blend instead of hard-edging.
fn terrain_vertex_color(map: &GameMap, gx: i32, gy: i32) -> [f32; 4] {
    let mut acc = [0.0f32; 3];
    let mut n = 0.0;
    for (dx, dy) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
        let c = terrain_color(map.get(gx + dx, gy + dy));
        acc[0] += c[0];
        acc[1] += c[1];
        acc[2] += c[2];
        n += 1.0;
    }
    [acc[0] / n, acc[1] / n, acc[2] / n, 1.0]
}

/// Muted, earthy base colours (linear-ish sRGB).
fn terrain_color(t: Terrain) -> [f32; 3] {
    match t {
        Terrain::DeepWater => [0.10, 0.16, 0.23],
        Terrain::Water => [0.17, 0.25, 0.32],
        Terrain::Plains => [0.42, 0.40, 0.26],
        Terrain::Grass => [0.30, 0.36, 0.20],
        Terrain::Forest => [0.20, 0.28, 0.16],
        Terrain::Hills => [0.40, 0.35, 0.27],
        Terrain::Mountain => [0.38, 0.37, 0.36],
    }
}
