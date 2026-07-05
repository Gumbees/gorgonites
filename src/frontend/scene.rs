//! The 3D scene: terrain mesh, water plane, sun, sky, and fog.
//!
//! Coordinate bridge — the sim is 2D `(x, y)` in "sim units" (TILE = 32 per
//! tile). We map that to Bevy's right-handed Y-up space as
//! `(x, elevation, y)`: the sim's Y axis becomes world Z, and terrain relief
//! becomes world Y. `WORLD` scales sim units down so the whole map is a
//! sensible number of Bevy units across.

use bevy::image::{
    ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::pbr::{CascadeShadowConfigBuilder, NotShadowCaster};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::Face;

use crate::game::{
    hash01, GameMap, Terrain, World as SimWorld, MAP_H, MAP_W, TILE,
};

use super::sim::Sim;
use super::AppState;

/// Load a colour (sRGB) texture set to tile/repeat across a surface.
pub fn load_tiled_srgb(asset_server: &AssetServer, path: &str) -> Handle<Image> {
    asset_server.load_with_settings(path.to_string(), |s: &mut ImageLoaderSettings| {
        s.sampler = repeat_sampler();
    })
}

/// Load a linear (non-sRGB) texture — normal/data maps must not be gamma-decoded.
pub fn load_tiled_linear(asset_server: &AssetServer, path: &str) -> Handle<Image> {
    asset_server.load_with_settings(path.to_string(), |s: &mut ImageLoaderSettings| {
        s.is_srgb = false;
        s.sampler = repeat_sampler();
    })
}

fn repeat_sampler() -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        ..Default::default()
    })
}

/// Shared horizon-haze colour so fog and clear colour agree with the sky.
pub const HORIZON: Color = Color::srgb(0.66, 0.70, 0.73);

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
pub struct Battlefield;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), setup_scene)
            .add_systems(OnExit(AppState::Playing), teardown_scene);
    }
}

fn teardown_scene(mut commands: Commands, roots: Query<Entity, With<Battlefield>>) {
    for e in &roots {
        commands.entity(e).despawn();
    }
}

fn setup_scene(
    mut commands: Commands,
    sim: Res<Sim>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // (map elevation is sampled inside build_terrain_mesh)

    // --- Sun: a low, warm key light with long, tightly-fitted shadows ------
    commands.spawn((
        Battlefield,
        DirectionalLight {
            color: Color::srgb(1.0, 0.93, 0.80),
            illuminance: 12_500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        // Cascades fitted to the RTS view distance keep shadow edges crisp.
        CascadeShadowConfigBuilder {
            num_cascades: 3,
            maximum_distance: 320.0,
            first_cascade_far_bound: 40.0,
            ..Default::default()
        }
        .build(),
        Transform::from_xyz(160.0, 150.0, 40.0)
            .looking_at(Vec3::new(120.0, 0.0, 120.0), Vec3::Y),
    ));

    // Sky-tinted ambient fill so shadowed faces read cool, not black.
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.55, 0.64, 0.80),
        brightness: 550.0,
        ..Default::default()
    });

    // --- Sky dome: a large inverted sphere textured with a real CC0 sky
    // HDRI (Poly Haven, equirectangular). Unlit + front-culled so the camera
    // sits inside it; the HDR's bright sun blooms via the camera's bloom.
    let sky_mesh = meshes.add(build_sky_mesh());
    let sky_tex = asset_server.load("hdri/kloofendal_sky_1k.hdr");
    let sky_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(sky_tex),
        unlit: true,
        cull_mode: Some(Face::Front), // render the inside of the dome
        fog_enabled: false,
        ..Default::default()
    });
    commands.spawn((
        Battlefield,
        Mesh3d(sky_mesh),
        MeshMaterial3d(sky_mat),
        NotShadowCaster,
        Transform::from_xyz(
            MAP_W as f32 * TILE * WORLD * 0.5,
            0.0,
            MAP_H as f32 * TILE * WORLD * 0.5,
        ),
    ));

    // --- Terrain mesh ------------------------------------------------------
    // A real tiling grass albedo + normal map, modulated per-vertex by the
    // biome colour so plains/forest/hills read distinctly while sharing one
    // detailed ground surface.
    let terrain_mesh = meshes.add(build_terrain_mesh(&sim.world));
    let terrain_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.1, 1.1, 1.1),
        base_color_texture: Some(load_tiled_srgb(&asset_server, "textures/ground/color.jpg")),
        normal_map_texture: Some(load_tiled_linear(&asset_server, "textures/ground/normal.jpg")),
        perceptual_roughness: 0.92,
        metallic: 0.0,
        ..Default::default()
    });
    commands.spawn((
        Battlefield,
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
        Battlefield,
        Mesh3d(water),
        MeshMaterial3d(water_mat),
        Transform::from_xyz(map_w / 2.0, -1.6 * WORLD, map_h / 2.0),
    ));

    scatter_vegetation(&mut commands, &sim.world, &mut meshes, &mut materials);
}

/// Scatter low-poly trees on forest tiles and boulders on mountains so the
/// map reads as a living landscape rather than a coloured grid. Static props
/// (spawned once, cleared on scene teardown) — no per-frame syncing.
fn scatter_vegetation(
    commands: &mut Commands,
    world: &SimWorld,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let map = &world.map;
    let trunk_mesh = meshes.add(Cylinder::new(0.12, 1.1));
    let canopy_mesh = meshes.add(Cone {
        radius: 1.0,
        height: 2.6,
    });
    let boulder_mesh = meshes.add(Sphere::new(0.7).mesh().ico(1).unwrap());

    let trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.20, 0.13),
        perceptual_roughness: 0.95,
        ..Default::default()
    });
    let canopy_mats: Vec<Handle<StandardMaterial>> = [
        [0.16, 0.28, 0.15],
        [0.20, 0.32, 0.17],
        [0.13, 0.24, 0.13],
    ]
    .iter()
    .map(|c| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(c[0], c[1], c[2]),
            perceptual_roughness: 0.9,
            ..Default::default()
        })
    })
    .collect();
    let boulder_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.34, 0.33),
        perceptual_roughness: 1.0,
        ..Default::default()
    });

    for ty in 0..MAP_H {
        for tx in 0..MAP_W {
            let terrain = map.get(tx, ty);
            let seed = map.seed;
            match terrain {
                Terrain::Forest => {
                    // Up to two trees per forest tile, hash-jittered.
                    for k in 0..2 {
                        if hash01(tx, ty, seed ^ (0xA1 + k)) > 0.55 {
                            continue;
                        }
                        let jx = (hash01(tx, ty, seed ^ (0x10 + k)) - 0.5) * TILE * 0.7;
                        let jy = (hash01(tx, ty, seed ^ (0x20 + k)) - 0.5) * TILE * 0.7;
                        let wx = (tx as f32 + 0.5) * TILE + jx;
                        let wy = (ty as f32 + 0.5) * TILE + jy;
                        let base = sim_to_world(map, wx, wy);
                        let s = 0.8 + hash01(tx, ty, seed ^ (0x30 + k)) * 0.7;
                        let ci = (hash01(tx, ty, seed ^ (0x40 + k)) * 3.0) as usize % 3;
                        // Trunk.
                        commands.spawn((
                            Battlefield,
                            Mesh3d(trunk_mesh.clone()),
                            MeshMaterial3d(trunk_mat.clone()),
                            Transform::from_translation(base + Vec3::Y * 0.55 * s)
                                .with_scale(Vec3::splat(s)),
                        ));
                        // Canopy.
                        commands.spawn((
                            Battlefield,
                            Mesh3d(canopy_mesh.clone()),
                            MeshMaterial3d(canopy_mats[ci].clone()),
                            Transform::from_translation(base + Vec3::Y * 2.1 * s)
                                .with_scale(Vec3::splat(s)),
                        ));
                    }
                }
                Terrain::Mountain => {
                    if hash01(tx, ty, seed ^ 0x77) < 0.4 {
                        let base = sim_to_world(map, (tx as f32 + 0.5) * TILE, (ty as f32 + 0.5) * TILE);
                        let s = 0.9 + hash01(tx, ty, seed ^ 0x88) * 1.2;
                        commands.spawn((
                            Battlefield,
                            Mesh3d(boulder_mesh.clone()),
                            MeshMaterial3d(boulder_mat.clone()),
                            Transform::from_translation(base + Vec3::Y * 0.3 * s)
                                .with_scale(Vec3::new(s, s * 0.7, s)),
                        ));
                    }
                }
                _ => {}
            }
        }
    }
}

/// A large UV sphere for the sky dome. Its equirectangular UVs line up with
/// the equirectangular sky HDRI applied as an unlit texture.
fn build_sky_mesh() -> Mesh {
    Sphere::new(1400.0).mesh().uv(48, 32)
}

/// Build a single terrain mesh with per-vertex colour by terrain class and
/// smooth normals, so the PBR sun shades ridges and valleys.
fn build_terrain_mesh(world: &SimWorld) -> Mesh {
    let map = &world.map;
    let w = MAP_W as usize;
    let h = MAP_H as usize;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((w + 1) * (h + 1));
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity((w + 1) * (h + 1));
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity((w + 1) * (h + 1));

    // One vertex per tile corner (grid is (w+1) x (h+1) vertices). The ground
    // texture tiles once per `TILE_REPEAT` tiles.
    const TILE_REPEAT: f32 = 3.0;
    for gy in 0..=h {
        for gx in 0..=w {
            let sx = gx as f32 * TILE;
            let sy = gy as f32 * TILE;
            let elev = map.elevation_world(sx, sy);
            positions.push([sx * WORLD, elev * WORLD, sy * WORLD]);
            colors.push(terrain_vertex_color(map, gx as i32, gy as i32));
            uvs.push([gx as f32 / TILE_REPEAT, gy as f32 / TILE_REPEAT]);
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();
    // Tangents are required for normal mapping; derived from UVs + normals.
    let _ = mesh.generate_tangents();
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
