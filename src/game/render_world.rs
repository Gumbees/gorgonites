//! Battlefield rendering.
//!
//! Art direction: grounded and muted, Company of Heroes rather than saturated
//! cartoon RTS. Earthy terrain with per-tile tonal variation, soft unit
//! shadows, tracers and drifting smoke, and Rise of Nations-style national
//! border tinting drawn straight onto the terrain.

use macroquad::prelude::*;

use crate::rendering::GameCamera;

use super::entities::*;
use super::mapgen::{hash01, Terrain, MAP_H, MAP_W, TILE};
use super::world::{ParticleKind, World};

/// Base colour per terrain type — muted, natural tones.
fn terrain_base(t: Terrain) -> Color {
    match t {
        Terrain::DeepWater => Color::from_rgba(26, 41, 58, 255),
        Terrain::Water => Color::from_rgba(43, 63, 82, 255),
        Terrain::Plains => Color::from_rgba(125, 116, 80, 255),
        Terrain::Grass => Color::from_rgba(96, 108, 66, 255),
        Terrain::Forest => Color::from_rgba(74, 88, 56, 255),
        Terrain::Hills => Color::from_rgba(118, 104, 82, 255),
        Terrain::Mountain => Color::from_rgba(104, 100, 96, 255),
    }
}

fn shade(c: Color, f: f32) -> Color {
    Color::new((c.r * f).min(1.0), (c.g * f).min(1.0), (c.b * f).min(1.0), c.a)
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

pub fn render_world(world: &World, camera: &GameCamera) {
    let top_left = camera.screen_to_world(vec2(0.0, 0.0));
    let bottom_right = camera.screen_to_world(vec2(screen_width(), screen_height()));
    let x0 = ((top_left.x / TILE).floor() as i32 - 1).max(0);
    let y0 = ((top_left.y / TILE).floor() as i32 - 1).max(0);
    let x1 = ((bottom_right.x / TILE).ceil() as i32 + 1).min(MAP_W);
    let y1 = ((bottom_right.y / TILE).ceil() as i32 + 1).min(MAP_H);

    draw_terrain(world, camera, x0, y0, x1, y1);
    draw_borders(world, camera, x0, y0, x1, y1);
    draw_buildings(world, camera);
    draw_units(world, camera);
    draw_particles(world, camera);
}

fn draw_terrain(world: &World, camera: &GameCamera, x0: i32, y0: i32, x1: i32, y1: i32) {
    let z = camera.zoom;
    let t = get_time() as f32;
    for y in y0..y1 {
        for x in x0..x1 {
            let terrain = world.map.get(x, y);
            let mut color = terrain_base(terrain);
            // Per-tile tonal jitter breaks up the flatness — reads as ground
            // texture at a distance.
            let jitter = 0.92 + hash01(x, y, 7) * 0.16;
            color = shade(color, jitter);
            if matches!(terrain, Terrain::Water | Terrain::DeepWater) {
                // Slow shimmering water.
                let wave = ((t * 0.8 + (x as f32) * 0.7 + (y as f32) * 1.3).sin() * 0.5 + 0.5)
                    * 0.08;
                color = shade(color, 1.0 + wave);
            }
            let p = camera.world_to_screen(vec2(x as f32 * TILE, y as f32 * TILE));
            let size = TILE * z + 1.0;
            draw_rectangle(p.x, p.y, size, size, color);

            // Terrain detailing.
            match terrain {
                Terrain::Forest => {
                    for i in 0..3 {
                        let ox = hash01(x, y, 30 + i) * TILE;
                        let oy = hash01(x, y, 60 + i) * TILE;
                        let c = camera
                            .world_to_screen(vec2(x as f32 * TILE + ox, y as f32 * TILE + oy));
                        let r = (4.0 + hash01(x, y, 90 + i) * 4.0) * z;
                        draw_circle(c.x + 1.5 * z, c.y + 2.0 * z, r, with_alpha(BLACK, 0.25));
                        draw_circle(
                            c.x,
                            c.y,
                            r,
                            shade(
                                Color::from_rgba(52, 70, 40, 255),
                                0.9 + hash01(x, y, 120 + i) * 0.25,
                            ),
                        );
                    }
                }
                Terrain::Mountain => {
                    let cx = (x as f32 + 0.5) * TILE;
                    let cy = (y as f32 + 0.5) * TILE;
                    let a = camera.world_to_screen(vec2(cx - 10.0, cy + 9.0));
                    let b = camera.world_to_screen(vec2(cx + 10.0, cy + 9.0));
                    let c = camera.world_to_screen(vec2(cx, cy - 11.0));
                    draw_triangle(a, b, c, Color::from_rgba(88, 84, 82, 255));
                    if hash01(x, y, 9) > 0.5 {
                        let snow_a = camera.world_to_screen(vec2(cx - 3.5, cy - 4.0));
                        let snow_b = camera.world_to_screen(vec2(cx + 3.5, cy - 4.0));
                        draw_triangle(snow_a, snow_b, c, Color::from_rgba(200, 202, 205, 255));
                    }
                }
                Terrain::Hills => {
                    let cx = (x as f32 + 0.5) * TILE;
                    let cy = (y as f32 + 0.5) * TILE;
                    let p1 = camera.world_to_screen(vec2(cx, cy));
                    draw_circle(p1.x, p1.y, 7.0 * z, shade(terrain_base(Terrain::Hills), 1.12));
                }
                _ => {}
            }

            if world.map.has_oil(x, y) {
                let c = camera
                    .world_to_screen(vec2((x as f32 + 0.5) * TILE, (y as f32 + 0.5) * TILE));
                draw_circle(c.x, c.y, 6.5 * z, Color::from_rgba(30, 28, 26, 255));
                draw_circle(c.x - 2.0 * z, c.y - 2.0 * z, 2.2 * z, Color::from_rgba(60, 56, 52, 255));
            }
        }
    }
}

/// National territory: a soft tint over owned tiles plus a hard border line
/// wherever ownership changes — the classic Rise of Nations map read.
fn draw_borders(world: &World, camera: &GameCamera, x0: i32, y0: i32, x1: i32, y1: i32) {
    let z = camera.zoom;
    for y in y0..y1 {
        for x in x0..x1 {
            let Some(owner) = world.tile_owner(x, y) else {
                continue;
            };
            let color = world.nations[owner as usize].color;
            let p = camera.world_to_screen(vec2(x as f32 * TILE, y as f32 * TILE));
            let size = TILE * z + 1.0;
            draw_rectangle(p.x, p.y, size, size, with_alpha(color, 0.09));

            let line = with_alpha(shade(color, 1.25), 0.85);
            let w = (2.0 * z).max(1.0);
            if world.tile_owner(x, y - 1) != Some(owner) {
                draw_line(p.x, p.y, p.x + size, p.y, w, line);
            }
            if world.tile_owner(x, y + 1) != Some(owner) {
                draw_line(p.x, p.y + size, p.x + size, p.y + size, w, line);
            }
            if world.tile_owner(x - 1, y) != Some(owner) {
                draw_line(p.x, p.y, p.x, p.y + size, w, line);
            }
            if world.tile_owner(x + 1, y) != Some(owner) {
                draw_line(p.x + size, p.y, p.x + size, p.y + size, w, line);
            }
        }
    }
}

fn draw_buildings(world: &World, camera: &GameCamera) {
    let z = camera.zoom;
    for b in &world.buildings {
        let nation_color = world.nations[b.nation].color;
        let he = b.half_extent();
        let p = camera.world_to_screen(b.pos - vec2(he, he));
        let s = he * 2.0 * z;

        // Ground shadow.
        draw_rectangle(p.x + 3.0 * z, p.y + 4.0 * z, s, s, with_alpha(BLACK, 0.25));

        let (wall, roof) = building_palette(b.kind);
        draw_rectangle(p.x, p.y, s, s, wall);

        match b.kind {
            BuildingKind::City => {
                // Keep + inner court + banner.
                draw_rectangle(p.x + s * 0.15, p.y + s * 0.15, s * 0.7, s * 0.7, shade(wall, 0.85));
                draw_rectangle(p.x + s * 0.3, p.y + s * 0.3, s * 0.4, s * 0.4, roof);
                draw_rectangle(p.x + s * 0.44, p.y - s * 0.18, s * 0.05, s * 0.24, shade(wall, 0.6));
                draw_rectangle(p.x + s * 0.49, p.y - s * 0.18, s * 0.22, s * 0.12, nation_color);
            }
            BuildingKind::Farm => {
                // Crop rows.
                for i in 0..4 {
                    let fy = p.y + s * (0.12 + i as f32 * 0.22);
                    draw_rectangle(p.x + s * 0.06, fy, s * 0.88, s * 0.1, shade(roof, 0.9 + (i % 2) as f32 * 0.15));
                }
            }
            _ => {
                // Gabled roof block.
                draw_rectangle(p.x + s * 0.1, p.y + s * 0.12, s * 0.8, s * 0.5, roof);
                draw_rectangle(p.x + s * 0.1, p.y + s * 0.62, s * 0.8, s * 0.26, shade(wall, 0.8));
                // Nation trim.
                draw_rectangle(p.x, p.y + s - 3.0 * z, s, 3.0 * z, with_alpha(nation_color, 0.9));
            }
        }

        if !b.is_complete() {
            // Scaffolding cross-hatch + build progress.
            draw_rectangle(p.x, p.y, s, s, with_alpha(BLACK, 0.35));
            draw_line(p.x, p.y, p.x + s, p.y + s, 2.0 * z, with_alpha(WHITE, 0.25));
            draw_line(p.x + s, p.y, p.x, p.y + s, 2.0 * z, with_alpha(WHITE, 0.25));
            draw_rectangle(p.x, p.y - 7.0 * z, s * b.construction, 4.0 * z, Color::from_rgba(210, 190, 120, 255));
            draw_rectangle_lines(p.x, p.y - 7.0 * z, s, 4.0 * z, 1.0, with_alpha(BLACK, 0.6));
        } else if b.hp < b.max_hp {
            let frac = (b.hp / b.max_hp).clamp(0.0, 1.0);
            let col = if frac > 0.6 {
                Color::from_rgba(120, 170, 90, 255)
            } else if frac > 0.3 {
                Color::from_rgba(210, 170, 70, 255)
            } else {
                Color::from_rgba(190, 70, 55, 255)
            };
            draw_rectangle(p.x, p.y - 7.0 * z, s * frac, 4.0 * z, col);
            draw_rectangle_lines(p.x, p.y - 7.0 * z, s, 4.0 * z, 1.0, with_alpha(BLACK, 0.6));
        }

        // Production progress ring under active queues.
        if b.is_complete() && !b.queue.is_empty() {
            draw_rectangle(p.x, p.y + s + 3.0 * z, s * 0.6, 3.0 * z, with_alpha(WHITE, 0.5));
        }
    }
}

fn building_palette(kind: BuildingKind) -> (Color, Color) {
    match kind {
        BuildingKind::City => (Color::from_rgba(140, 133, 120, 255), Color::from_rgba(112, 84, 62, 255)),
        BuildingKind::Farm => (Color::from_rgba(122, 108, 70, 255), Color::from_rgba(150, 132, 74, 255)),
        BuildingKind::LumberCamp => (Color::from_rgba(110, 92, 66, 255), Color::from_rgba(86, 68, 48, 255)),
        BuildingKind::Mine => (Color::from_rgba(120, 116, 110, 255), Color::from_rgba(84, 82, 80, 255)),
        BuildingKind::Market => (Color::from_rgba(150, 130, 96, 255), Color::from_rgba(160, 110, 70, 255)),
        BuildingKind::University => (Color::from_rgba(160, 152, 136, 255), Color::from_rgba(94, 102, 120, 255)),
        BuildingKind::Barracks => (Color::from_rgba(120, 110, 96, 255), Color::from_rgba(96, 78, 60, 255)),
        BuildingKind::OilWell => (Color::from_rgba(90, 86, 82, 255), Color::from_rgba(50, 48, 46, 255)),
    }
}

fn draw_units(world: &World, camera: &GameCamera) {
    let z = camera.zoom;
    for u in &world.units {
        let nation_color = world.nations[u.nation].color;
        let p = camera.world_to_screen(u.pos);
        let r = u.radius() * z;

        // Soft shadow.
        draw_circle(p.x + 1.5 * z, p.y + 2.5 * z, r, with_alpha(BLACK, 0.3));

        // Body: muted uniform in the nation's colour.
        let body = shade(nation_color, 0.9);
        match u.kind {
            UnitKind::Citizen => {
                draw_circle(p.x, p.y, r, Color::from_rgba(168, 152, 122, 255));
                draw_circle(p.x, p.y - r * 0.5, r * 0.5, shade(nation_color, 1.1));
            }
            UnitKind::Cavalry => {
                let (dx, dy) = (u.facing.cos(), u.facing.sin());
                draw_circle(p.x, p.y, r, shade(body, 0.85));
                draw_circle(p.x + dx * r * 0.7, p.y + dy * r * 0.7, r * 0.6, body);
            }
            UnitKind::Siege => {
                draw_rectangle(p.x - r, p.y - r * 0.7, r * 2.0, r * 1.4, shade(body, 0.8));
                draw_circle(p.x - r * 0.6, p.y + r * 0.7, r * 0.35, Color::from_rgba(40, 38, 36, 255));
                draw_circle(p.x + r * 0.6, p.y + r * 0.7, r * 0.35, Color::from_rgba(40, 38, 36, 255));
            }
            _ => {
                draw_circle(p.x, p.y, r, body);
                draw_circle(p.x, p.y - r * 0.35, r * 0.45, Color::from_rgba(196, 178, 148, 255));
            }
        }

        // Weapon line pointing at the facing.
        if u.kind.is_military() {
            let (dx, dy) = (u.facing.cos(), u.facing.sin());
            let len = if u.kind == UnitKind::Siege { r * 2.0 } else { r * 1.6 };
            draw_line(
                p.x,
                p.y,
                p.x + dx * len,
                p.y + dy * len,
                (1.6 * z).max(1.0),
                Color::from_rgba(56, 52, 48, 255),
            );
        }

        // Health bar when hurt.
        if u.hp < u.stats.max_hp {
            let frac = (u.hp / u.stats.max_hp).clamp(0.0, 1.0);
            let w = r * 2.4;
            let col = if frac > 0.6 {
                Color::from_rgba(120, 170, 90, 255)
            } else if frac > 0.3 {
                Color::from_rgba(210, 170, 70, 255)
            } else {
                Color::from_rgba(190, 70, 55, 255)
            };
            draw_rectangle(p.x - w / 2.0, p.y - r - 6.0 * z, w, 2.5 * z, with_alpha(BLACK, 0.5));
            draw_rectangle(p.x - w / 2.0, p.y - r - 6.0 * z, w * frac, 2.5 * z, col);
        }
    }
}

/// Draw selection rings for the given unit ids and building id.
pub fn draw_selection(world: &World, camera: &GameCamera, units: &[Id], building: Option<Id>) {
    let z = camera.zoom;
    let ring = Color::from_rgba(230, 240, 220, 210);
    for id in units {
        if let Some(u) = world.unit(*id) {
            let p = camera.world_to_screen(u.pos);
            draw_circle_lines(p.x, p.y + 1.0 * z, u.radius() * z + 3.0 * z, (1.5 * z).max(1.0), ring);
        }
    }
    if let Some(id) = building {
        if let Some(b) = world.building(id) {
            let he = b.half_extent();
            let p = camera.world_to_screen(b.pos - vec2(he, he));
            let s = he * 2.0 * z;
            draw_rectangle_lines(p.x - 2.0, p.y - 2.0, s + 4.0, s + 4.0, (2.0 * z).max(1.5), ring);
        }
    }
}

fn draw_particles(world: &World, camera: &GameCamera) {
    let z = camera.zoom;
    for p in &world.particles {
        let frac = (p.life / p.max_life).clamp(0.0, 1.0);
        let sp = camera.world_to_screen(p.pos);
        match p.kind {
            ParticleKind::Tracer { to } => {
                let ep = camera.world_to_screen(to);
                draw_line(sp.x, sp.y, ep.x, ep.y, (p.size * z).max(1.0), with_alpha(p.color, frac));
            }
            ParticleKind::Flash => {
                draw_circle(sp.x, sp.y, p.size * z * (2.0 - frac), with_alpha(p.color, frac * 0.9));
            }
            ParticleKind::Smoke => {
                draw_circle(sp.x, sp.y, p.size * z, with_alpha(p.color, frac * 0.55));
            }
            ParticleKind::Blood | ParticleKind::Spark => {
                draw_circle(sp.x, sp.y, p.size * z, with_alpha(p.color, frac));
            }
        }
    }
}

/// Ghost preview while placing a building.
pub fn draw_placement_ghost(
    world: &World,
    camera: &GameCamera,
    nation: usize,
    kind: BuildingKind,
    tile: (i32, i32),
) {
    let z = camera.zoom;
    let ok = world.can_place(nation, kind, tile).is_ok();
    let fp = kind.footprint() as f32;
    let p = camera.world_to_screen(vec2(tile.0 as f32 * TILE, tile.1 as f32 * TILE));
    let s = fp * TILE * z;
    let color = if ok {
        Color::from_rgba(120, 200, 120, 90)
    } else {
        Color::from_rgba(210, 80, 70, 90)
    };
    draw_rectangle(p.x, p.y, s, s, color);
    draw_rectangle_lines(p.x, p.y, s, s, 2.0, with_alpha(color, 0.9));
}

