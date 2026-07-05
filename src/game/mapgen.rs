//! Procedural battlefield generation.
//!
//! Layered value noise produces continents of plains, grassland, forest,
//! hills, and mountains with lakes — plus scattered oil deposits that only
//! matter once a nation industrialises.

pub const MAP_W: i32 = 96;
pub const MAP_H: i32 = 96;
pub const TILE: f32 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    DeepWater,
    Water,
    Plains,
    Grass,
    Forest,
    Hills,
    Mountain,
}

impl Terrain {
    pub fn passable(&self) -> bool {
        !matches!(self, Terrain::DeepWater | Terrain::Water | Terrain::Mountain)
    }

    pub fn buildable(&self) -> bool {
        matches!(self, Terrain::Plains | Terrain::Grass | Terrain::Hills)
    }
}

/// Deterministic per-cell hash in [0, 1).
pub fn hash01(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise(fx: f32, fy: f32, seed: u32) -> f32 {
    let x0 = fx.floor() as i32;
    let y0 = fy.floor() as i32;
    let tx = smooth(fx - x0 as f32);
    let ty = smooth(fy - y0 as f32);
    let a = hash01(x0, y0, seed);
    let b = hash01(x0 + 1, y0, seed);
    let c = hash01(x0, y0 + 1, seed);
    let d = hash01(x0 + 1, y0 + 1, seed);
    let top = a + (b - a) * tx;
    let bottom = c + (d - c) * tx;
    top + (bottom - top) * ty
}

fn fbm(fx: f32, fy: f32, seed: u32) -> f32 {
    let mut total = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for octave in 0..4 {
        total += value_noise(fx * freq, fy * freq, seed.wrapping_add(octave * 7919)) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    total / norm
}

pub struct GameMap {
    pub w: i32,
    pub h: i32,
    terrain: Vec<Terrain>,
    oil: Vec<bool>,
}

impl GameMap {
    pub fn generate(seed: u32, start_areas: &[(i32, i32)]) -> Self {
        let mut terrain = vec![Terrain::Plains; (MAP_W * MAP_H) as usize];
        let mut oil = vec![false; (MAP_W * MAP_H) as usize];

        for y in 0..MAP_H {
            for x in 0..MAP_W {
                let fx = x as f32;
                let fy = y as f32;
                let elev = fbm(fx / 18.0, fy / 18.0, seed);
                let moisture = fbm(fx / 13.0, fy / 13.0, seed.wrapping_add(101));
                let t = if elev < 0.30 {
                    Terrain::DeepWater
                } else if elev < 0.37 {
                    Terrain::Water
                } else if elev > 0.78 {
                    Terrain::Mountain
                } else if elev > 0.69 {
                    Terrain::Hills
                } else if moisture > 0.62 {
                    Terrain::Forest
                } else if moisture > 0.45 {
                    Terrain::Grass
                } else {
                    Terrain::Plains
                };
                terrain[(y * MAP_W + x) as usize] = t;
            }
        }

        // Clear generous starting areas so both nations get workable land.
        for &(sx, sy) in start_areas {
            for y in (sy - 7).max(0)..(sy + 8).min(MAP_H) {
                for x in (sx - 7).max(0)..(sx + 8).min(MAP_W) {
                    let dx = (x - sx) as f32;
                    let dy = (y - sy) as f32;
                    let d = (dx * dx + dy * dy).sqrt();
                    let idx = (y * MAP_W + x) as usize;
                    if d < 5.0 {
                        terrain[idx] = if hash01(x, y, seed ^ 0xBEEF) > 0.5 {
                            Terrain::Grass
                        } else {
                            Terrain::Plains
                        };
                    } else if d < 7.5 && !terrain[idx].passable() {
                        terrain[idx] = Terrain::Plains;
                    }
                }
            }
            // Guarantee nearby forest (timber) and hills (metal) for each start.
            let fx = (sx + 5).clamp(1, MAP_W - 2);
            let fy = (sy - 5).clamp(1, MAP_H - 2);
            for dy in 0..3 {
                for dx in 0..3 {
                    terrain[((fy + dy) * MAP_W + fx + dx) as usize] = Terrain::Forest;
                }
            }
            let hx = (sx - 6).clamp(1, MAP_W - 3);
            let hy = (sy + 5).clamp(1, MAP_H - 3);
            for dy in 0..2 {
                for dx in 0..2 {
                    terrain[((hy + dy) * MAP_W + hx + dx) as usize] = Terrain::Hills;
                }
            }
        }

        // Scatter oil deposits on open land, away from the coasts.
        let mut placed = 0;
        let mut attempt = 0;
        while placed < 14 && attempt < 4000 {
            attempt += 1;
            let x = 4 + (hash01(attempt, 17, seed ^ 0x011) * (MAP_W - 8) as f32) as i32;
            let y = 4 + (hash01(attempt, 91, seed ^ 0x0FA) * (MAP_H - 8) as f32) as i32;
            let idx = (y * MAP_W + x) as usize;
            if matches!(terrain[idx], Terrain::Plains | Terrain::Grass) && !oil[idx] {
                oil[idx] = true;
                // Deposits come in small patches.
                if x + 1 < MAP_W {
                    oil[idx + 1] = matches!(terrain[idx + 1], Terrain::Plains | Terrain::Grass);
                }
                placed += 1;
            }
        }

        Self {
            w: MAP_W,
            h: MAP_H,
            terrain,
            oil,
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    pub fn get(&self, x: i32, y: i32) -> Terrain {
        if !self.in_bounds(x, y) {
            return Terrain::DeepWater;
        }
        self.terrain[(y * self.w + x) as usize]
    }

    pub fn has_oil(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.oil[(y * self.w + x) as usize]
    }

    pub fn tile_at_world(&self, wx: f32, wy: f32) -> (i32, i32) {
        ((wx / TILE).floor() as i32, (wy / TILE).floor() as i32)
    }

    pub fn passable_world(&self, wx: f32, wy: f32) -> bool {
        let (tx, ty) = self.tile_at_world(wx, wy);
        self.get(tx, ty).passable()
    }

    /// Is there a tile of the given terrain within `radius` tiles?
    pub fn terrain_near(&self, x: i32, y: i32, radius: i32, want: &[Terrain]) -> bool {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if want.contains(&self.get(x + dx, y + dy)) {
                    return true;
                }
            }
        }
        false
    }
}
