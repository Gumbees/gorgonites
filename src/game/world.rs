//! The live battlefield simulation.
//!
//! Rise of Nations rules implemented here:
//! - **National borders**: every city projects territory; the border grid is
//!   the union of city influence, and you may only build inside your own.
//! - **Attrition**: enemy units bleed hit points every second they stand on
//!   your soil, scaling with your age.
//! - **City capture**: cities are never destroyed — reduce one to rubble and
//!   it changes flags. Lose your capital and a countdown starts; if it runs
//!   out, your nation falls.
//! - **Commerce-capped economy**: citizens staff economic buildings and
//!   produce continuous resource rates, clamped per resource by the age's
//!   commerce cap.
//! - **Age advancement**: researched at a city, re-skins and upgrades every
//!   unit line and pushes your borders outward.

use ::rand::Rng;
use bevy::math::{vec2, Vec2};

use crate::systems::rts::{
    calculate_damage, commerce_cap, ArmorType, Cost, Resource, Stockpile,
};

use super::ai_nation::{ai_tick, AiState};
use super::entities::*;
use super::mapgen::{GameMap, Terrain, MAP_H, MAP_W, TILE};

pub const CAPITAL_COUNTDOWN: f32 = 60.0;
const MIN_CITY_DIST_TILES: i32 = 8;

// ---------------------------------------------------------------------------
// Nations
// ---------------------------------------------------------------------------

/// Linear-ish RGB triple; the frontend maps this to engine colours.
pub type Rgb = [f32; 3];
/// RGBA quad for particle tinting.
pub type Rgba = [f32; 4];

pub struct Nation {
    pub name: String,
    pub color: Rgb,
    /// 0-based age index into `Era::ALL`.
    pub age: usize,
    pub stockpile: Stockpile,
    /// Smoothed income per second, for the HUD.
    pub income: [f32; 6],
    /// True where income hit the commerce cap last tick.
    pub capped: [bool; 6],
    pub pop: i32,
    pub pop_cap: i32,
    pub is_ai: bool,
    pub defeated: bool,
    /// City id of this nation's capital.
    pub capital: Id,
    /// Seconds until defeat while the capital is in enemy hands.
    pub capital_timer: Option<f32>,
    pub kills: u32,
}

// ---------------------------------------------------------------------------
// Particles (tracers, smoke, muzzle flashes — the CoH dressing)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleKind {
    Smoke,
    Flash,
    Tracer { to: Vec2 },
    Blood,
    Spark,
}

pub struct Particle {
    /// Stable id so a frontend can sync engine entities to particles.
    pub id: Id,
    pub kind: ParticleKind,
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Rgba,
}

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

/// What a unit decided to do this frame (planned immutably, applied mutably).
struct UnitPlan {
    new_pos: Option<Vec2>,
    facing: Option<f32>,
    new_order: Option<Order>,
    attack_unit: Option<Id>,
    attack_building: Option<Id>,
}

pub struct World {
    pub map: GameMap,
    pub nations: Vec<Nation>,
    pub units: Vec<Unit>,
    pub buildings: Vec<Building>,
    /// Per-tile national ownership (index into `nations`).
    pub owner: Vec<Option<u8>>,
    pub particles: Vec<Particle>,
    pub game_time: f32,
    pub winner: Option<usize>,
    /// Bumped whenever the border grid is recomputed; frontends watch this
    /// to know when to rebuild territory overlays.
    pub border_version: u64,
    next_id: Id,
    borders_dirty: bool,
    econ_accum: f32,
    attrition_accum: f32,
    ai: Vec<AiState>,
}

impl World {
    /// A world on the fixed default map. Deterministic (seed `0xC0FFEE`) — used
    /// by the sim tests and as a throwaway placeholder before a game starts.
    pub fn new() -> Self {
        Self::with_seed(0xC0FFEE)
    }

    /// A world on a fresh, randomly seeded procedural map — a new battlefield
    /// every time a scenario begins.
    pub fn new_random() -> Self {
        Self::with_seed(::rand::random::<u32>())
    }

    /// Build a world whose terrain, rivers, and biomes are generated from
    /// `seed`. Same seed in, same map out.
    pub fn with_seed(seed: u32) -> Self {
        let player_start = (20, 48);
        let enemy_start = (76, 48);
        let map = GameMap::generate(seed, &[player_start, enemy_start]);

        let mut world = Self {
            map,
            nations: Vec::new(),
            units: Vec::new(),
            buildings: Vec::new(),
            owner: vec![None; (MAP_W * MAP_H) as usize],
            particles: Vec::new(),
            game_time: 0.0,
            winner: None,
            border_version: 0,
            next_id: 1,
            borders_dirty: true,
            econ_accum: 0.0,
            attrition_accum: 0.0,
            ai: vec![AiState::default(), AiState::default()],
        };

        world.nations.push(Nation {
            name: "Federation".to_string(),
            color: [0.31, 0.46, 0.66],
            age: 0,
            stockpile: Stockpile::starting(),
            income: [0.0; 6],
            capped: [false; 6],
            pop: 0,
            pop_cap: 0,
            is_ai: false,
            defeated: false,
            capital: 0,
            capital_timer: None,
            kills: 0,
        });
        world.nations.push(Nation {
            name: "Crimson Pact".to_string(),
            color: [0.62, 0.24, 0.21],
            age: 0,
            stockpile: Stockpile::starting(),
            income: [0.0; 6],
            capped: [false; 6],
            pop: 0,
            pop_cap: 0,
            is_ai: true,
            defeated: false,
            capital: 0,
            capital_timer: None,
            kills: 0,
        });

        for (nation, (sx, sy)) in [(0usize, player_start), (1usize, enemy_start)] {
            let city = world.place_building(nation, BuildingKind::City, (sx - 1, sy - 1), true);
            world.nations[nation].capital = city;
            let centre = vec2(sx as f32 * TILE, sy as f32 * TILE);
            for i in 0..4 {
                let angle = i as f32 * std::f32::consts::TAU / 4.0;
                let pos = centre + vec2(angle.cos(), angle.sin()) * TILE * 2.5;
                world.spawn_unit(nation, UnitKind::Citizen, pos);
            }
        }

        world.recompute_borders();
        world.recount_population();
        world
    }

    fn alloc_id(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    // -- lookups ------------------------------------------------------------

    pub fn unit(&self, id: Id) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }

    pub fn building(&self, id: Id) -> Option<&Building> {
        self.buildings.iter().find(|b| b.id == id)
    }

    pub fn building_mut(&mut self, id: Id) -> Option<&mut Building> {
        self.buildings.iter_mut().find(|b| b.id == id)
    }

    pub fn tile_owner(&self, x: i32, y: i32) -> Option<u8> {
        if x < 0 || y < 0 || x >= MAP_W || y >= MAP_H {
            return None;
        }
        self.owner[(y * MAP_W + x) as usize]
    }

    /// Living units of a kind owned by a nation (for ramping costs).
    pub fn count_units(&self, nation: usize, kind: UnitKind) -> usize {
        self.units
            .iter()
            .filter(|u| u.nation == nation && u.kind == kind)
            .count()
    }

    pub fn count_buildings(&self, nation: usize, kind: BuildingKind) -> usize {
        self.buildings
            .iter()
            .filter(|b| b.nation == nation && b.kind == kind)
            .count()
    }

    /// Citizens currently working (or en route to work) at a building.
    pub fn workers_at(&self, building: Id) -> usize {
        self.units
            .iter()
            .filter(|u| matches!(u.order, Order::Work { building: b } if b == building))
            .count()
    }

    // -- spawning & placement ------------------------------------------------

    pub fn spawn_unit(&mut self, nation: usize, kind: UnitKind, pos: Vec2) -> Id {
        let id = self.alloc_id();
        let stats = unit_stats(kind, self.nations[nation].age);
        self.units.push(Unit {
            id,
            nation,
            kind,
            pos,
            facing: 0.0,
            hp: stats.max_hp,
            stats,
            order: Order::Idle,
            cooldown: 0.0,
        });
        id
    }

    /// Validate a building placement under Rise of Nations territory rules.
    pub fn can_place(
        &self,
        nation: usize,
        kind: BuildingKind,
        tile: (i32, i32),
    ) -> Result<(), &'static str> {
        let fp = kind.footprint();
        if self.nations[nation].age < kind.min_age() {
            return Err("Requires a later age");
        }
        for dy in 0..fp {
            for dx in 0..fp {
                let (x, y) = (tile.0 + dx, tile.1 + dy);
                if !self.map.in_bounds(x, y) {
                    return Err("Out of bounds");
                }
                if !self.map.get(x, y).buildable() {
                    return Err("Terrain is unbuildable");
                }
                match kind {
                    // Cities push into neutral land; everything else must sit
                    // inside your national borders — the RoN rule.
                    BuildingKind::City => {
                        if let Some(o) = self.tile_owner(x, y) {
                            if o as usize != nation {
                                return Err("Inside enemy territory");
                            }
                        }
                    }
                    _ => {
                        if self.tile_owner(x, y) != Some(nation as u8) {
                            return Err("Must be inside your borders");
                        }
                    }
                }
            }
        }
        // No overlapping footprints (with a 1-tile buffer).
        for b in &self.buildings {
            let bfp = b.kind.footprint();
            if tile.0 < b.tile.0 + bfp + 1
                && b.tile.0 < tile.0 + fp + 1
                && tile.1 < b.tile.1 + bfp + 1
                && b.tile.1 < tile.1 + fp + 1
            {
                return Err("Too close to another building");
            }
        }
        match kind {
            BuildingKind::City => {
                for b in &self.buildings {
                    if b.kind == BuildingKind::City {
                        let dx = b.tile.0 - tile.0;
                        let dy = b.tile.1 - tile.1;
                        if dx * dx + dy * dy < MIN_CITY_DIST_TILES * MIN_CITY_DIST_TILES {
                            return Err("Too close to another city");
                        }
                    }
                }
            }
            BuildingKind::LumberCamp => {
                if !self.map.terrain_near(tile.0, tile.1, 3, &[Terrain::Forest]) {
                    return Err("Needs forest nearby");
                }
            }
            BuildingKind::Mine => {
                if !self
                    .map
                    .terrain_near(tile.0, tile.1, 3, &[Terrain::Hills, Terrain::Mountain])
                {
                    return Err("Needs hills or mountains nearby");
                }
            }
            BuildingKind::OilWell => {
                let mut on_oil = false;
                for dy in 0..fp {
                    for dx in 0..fp {
                        if self.map.has_oil(tile.0 + dx, tile.1 + dy) {
                            on_oil = true;
                        }
                    }
                }
                if !on_oil {
                    return Err("Must sit on an oil deposit");
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Insert a building; `completed` skips construction (starting cities).
    pub fn place_building(
        &mut self,
        nation: usize,
        kind: BuildingKind,
        tile: (i32, i32),
        completed: bool,
    ) -> Id {
        let id = self.alloc_id();
        let fp = kind.footprint() as f32;
        let pos = vec2(
            (tile.0 as f32 + fp / 2.0) * TILE,
            (tile.1 as f32 + fp / 2.0) * TILE,
        );
        self.buildings.push(Building {
            id,
            nation,
            kind,
            tile,
            pos,
            hp: if completed { kind.max_hp() } else { kind.max_hp() * 0.1 },
            max_hp: kind.max_hp(),
            construction: if completed { 1.0 } else { 0.0 },
            queue: Vec::new(),
            queue_progress: 0.0,
            last_attacker: None,
            smoke_timer: 0.0,
        });
        if kind == BuildingKind::City {
            self.borders_dirty = true;
        }
        id
    }

    /// Try to pay for and enqueue a production item. Returns an error string
    /// for the HUD toast when it can't happen.
    pub fn try_enqueue(&mut self, building_id: Id, item: QueueItem) -> Result<(), String> {
        let (nation_idx, kind, complete, queue_len) = match self.building(building_id) {
            Some(b) => (b.nation, b.kind, b.is_complete(), b.queue.len()),
            None => return Err("Building gone".into()),
        };
        if !complete {
            return Err("Still under construction".into());
        }
        if queue_len >= 5 {
            return Err("Queue is full".into());
        }
        let cost = match item {
            QueueItem::Unit(u) => {
                match (kind, u) {
                    (BuildingKind::City, UnitKind::Citizen) => {}
                    (BuildingKind::Barracks, k) if k.is_military() => {}
                    _ => return Err("Can't train that here".into()),
                }
                let nation = &self.nations[nation_idx];
                if nation.pop + unit_stats(u, nation.age).pop > nation.pop_cap {
                    return Err("Population limit reached — build more cities".into());
                }
                unit_ramped_cost(u, self.count_units(nation_idx, u))
            }
            QueueItem::AgeUp => {
                if kind != BuildingKind::City {
                    return Err("Ages are researched at a city".into());
                }
                let nation = &self.nations[nation_idx];
                if nation.age >= 7 {
                    return Err("Already in the final age".into());
                }
                if self
                    .buildings
                    .iter()
                    .any(|b| b.nation == nation_idx && b.queue.contains(&QueueItem::AgeUp))
                {
                    return Err("Age research already underway".into());
                }
                age_up_cost(nation.age + 1)
            }
        };
        if !self.nations[nation_idx].stockpile.pay(&cost) {
            return Err(format!("Not enough resources ({})", cost.describe()));
        }
        self.building_mut(building_id).unwrap().queue.push(item);
        Ok(())
    }

    // -- borders --------------------------------------------------------------

    fn city_radius(&self, nation: usize, is_capital: bool) -> f32 {
        let age = self.nations[nation].age as f32;
        6.0 + age + if is_capital { 2.0 } else { 0.0 }
    }

    /// Rebuild the national border grid: each tile belongs to the nation of
    /// the nearest city whose influence radius covers it.
    pub fn recompute_borders(&mut self) {
        self.owner.iter_mut().for_each(|o| *o = None);
        let cities: Vec<(usize, (i32, i32), f32)> = self
            .buildings
            .iter()
            .filter(|b| b.kind == BuildingKind::City && b.is_complete())
            .map(|b| {
                let is_capital = self.nations[b.nation].capital == b.id;
                let fp = b.kind.footprint();
                let cx = b.tile.0 + fp / 2;
                let cy = b.tile.1 + fp / 2;
                (b.nation, (cx, cy), self.city_radius(b.nation, is_capital))
            })
            .collect();

        for y in 0..MAP_H {
            for x in 0..MAP_W {
                let mut best: Option<(usize, f32)> = None;
                for &(nation, (cx, cy), radius) in &cities {
                    let dx = (x - cx) as f32;
                    let dy = (y - cy) as f32;
                    let d = (dx * dx + dy * dy).sqrt();
                    if d <= radius && best.map_or(true, |(_, bd)| d < bd) {
                        best = Some((nation, d));
                    }
                }
                self.owner[(y * MAP_W + x) as usize] = best.map(|(n, _)| n as u8);
            }
        }
        self.borders_dirty = false;
        self.border_version += 1;
    }

    fn recount_population(&mut self) {
        for (i, nation) in self.nations.iter_mut().enumerate() {
            nation.pop = 0;
            let cities = self
                .buildings
                .iter()
                .filter(|b| b.nation == i && b.kind == BuildingKind::City && b.is_complete())
                .count() as i32;
            nation.pop_cap = 20 + 15 * cities + 5 * nation.age as i32;
            for u in &self.units {
                if u.nation == i {
                    nation.pop += u.stats.pop;
                }
            }
        }
    }

    // -- main update ------------------------------------------------------------

    pub fn update(&mut self, dt: f32) {
        if self.winner.is_some() {
            self.update_particles(dt);
            return;
        }
        self.game_time += dt;

        self.update_construction(dt);
        self.update_production(dt);

        self.econ_accum += dt;
        if self.econ_accum >= 0.25 {
            let tick = self.econ_accum;
            self.econ_accum = 0.0;
            self.economy_tick(tick);
        }

        let plans = self.plan_units(dt);
        self.apply_plans(plans, dt);
        self.separate_units();

        self.attrition_accum += dt;
        if self.attrition_accum >= 1.0 {
            self.attrition_accum -= 1.0;
            self.attrition_tick();
        }

        self.resolve_deaths();

        // AI nations think with their state temporarily lifted out to keep
        // the borrow checker happy.
        let mut ai = std::mem::take(&mut self.ai);
        for (i, state) in ai.iter_mut().enumerate() {
            if self.nations.get(i).map_or(false, |n| n.is_ai && !n.defeated) {
                ai_tick(self, i, state, dt);
            }
        }
        self.ai = ai;

        if self.borders_dirty {
            self.recompute_borders();
        }
        self.recount_population();
        self.update_capitals(dt);
        self.update_particles(dt);
        self.check_victory();
    }

    fn update_construction(&mut self, dt: f32) {
        let mut dirty = false;
        for b in &mut self.buildings {
            if b.construction < 1.0 {
                let rate = dt / b.kind.build_time();
                b.construction = (b.construction + rate).min(1.0);
                b.hp = (b.hp + b.max_hp * 0.9 * rate).min(b.max_hp);
                if b.construction >= 1.0 && b.kind == BuildingKind::City {
                    dirty = true;
                }
            }
        }
        if dirty {
            self.borders_dirty = true;
        }
    }

    fn update_production(&mut self, dt: f32) {
        let mut spawns: Vec<(usize, UnitKind, Vec2)> = Vec::new();
        let mut age_ups: Vec<usize> = Vec::new();

        for b in &mut self.buildings {
            if !b.is_complete() || b.queue.is_empty() {
                b.queue_progress = 0.0;
                continue;
            }
            let needed = match b.queue[0] {
                QueueItem::Unit(u) => unit_stats(u, self.nations[b.nation].age).train_time,
                QueueItem::AgeUp => 25.0,
            };
            b.queue_progress += dt;
            if b.queue_progress >= needed {
                b.queue_progress = 0.0;
                match b.queue.remove(0) {
                    QueueItem::Unit(u) => {
                        let rally = b.pos + vec2(0.0, b.half_extent() + 20.0);
                        spawns.push((b.nation, u, rally));
                    }
                    QueueItem::AgeUp => age_ups.push(b.nation),
                }
            }
        }

        for (nation, kind, pos) in spawns {
            self.spawn_unit(nation, kind, pos);
        }
        for nation in age_ups {
            let n = &mut self.nations[nation];
            n.age = (n.age + 1).min(7);
            self.borders_dirty = true;
        }
    }

    fn economy_tick(&mut self, dt: f32) {
        let mut raw: Vec<[f32; 6]> = vec![[0.0; 6]; self.nations.len()];

        // City trickle: a functioning city produces a little food and wealth.
        for b in &self.buildings {
            if !b.is_complete() {
                continue;
            }
            match b.kind.output() {
                Some((resource, per_worker, slots)) => {
                    let workers = self.workers_at(b.id).min(slots);
                    raw[b.nation][resource.index()] += per_worker * workers as f32;
                }
                None => {
                    if b.kind == BuildingKind::City {
                        raw[b.nation][Resource::Wealth.index()] += 0.4;
                        raw[b.nation][Resource::Food.index()] += 0.2;
                    }
                }
            }
        }

        for (i, nation) in self.nations.iter_mut().enumerate() {
            let cap = commerce_cap(nation.age);
            for r in Resource::ALL {
                let idx = r.index();
                let rate = raw[i][idx];
                let clamped = rate.min(cap);
                nation.capped[idx] = rate > cap + 0.01;
                nation.income[idx] = clamped;
                nation.stockpile.add(r, clamped * dt);
            }
        }
    }

    // -- unit simulation ----------------------------------------------------

    fn plan_units(&self, dt: f32) -> Vec<UnitPlan> {
        self.units
            .iter()
            .map(|u| self.plan_one(u, dt))
            .collect()
    }

    fn plan_one(&self, u: &Unit, dt: f32) -> UnitPlan {
        let mut plan = UnitPlan {
            new_pos: None,
            facing: None,
            new_order: None,
            attack_unit: None,
            attack_building: None,
        };

        match u.order {
            Order::Idle => {
                if u.kind.is_military() {
                    if let Some(t) = self.nearest_enemy_unit(u, u.stats.aggro_range) {
                        plan.new_order = Some(Order::AttackUnit(t));
                    } else if let Some(t) = self.nearest_enemy_building(u, u.stats.aggro_range) {
                        plan.new_order = Some(Order::AttackBuilding(t));
                    }
                }
            }
            Order::Move { dest, aggro } => {
                if aggro && u.kind.is_military() {
                    if let Some(t) = self.nearest_enemy_unit(u, u.stats.aggro_range) {
                        plan.new_order = Some(Order::AttackUnit(t));
                        return plan;
                    }
                }
                let delta = dest - u.pos;
                if delta.length() < 6.0 {
                    plan.new_order = Some(Order::Idle);
                } else {
                    let step = self.step_toward(u.pos, dest, u.stats.speed * dt);
                    plan.facing = Some(delta.y.atan2(delta.x));
                    plan.new_pos = Some(step);
                }
            }
            Order::AttackUnit(target) => match self.unit(target) {
                Some(t) if t.hp > 0.0 => {
                    let delta = t.pos - u.pos;
                    let dist = delta.length();
                    plan.facing = Some(delta.y.atan2(delta.x));
                    if dist > u.stats.range + t.radius() {
                        plan.new_pos = Some(self.step_toward(u.pos, t.pos, u.stats.speed * dt));
                    } else if u.cooldown <= 0.0 {
                        plan.attack_unit = Some(target);
                    }
                }
                _ => plan.new_order = Some(Order::Idle),
            },
            Order::AttackBuilding(target) => match self.building(target) {
                Some(b) if b.hp > 0.0 && b.nation != u.nation => {
                    let delta = b.pos - u.pos;
                    let dist = delta.length();
                    plan.facing = Some(delta.y.atan2(delta.x));
                    if dist > u.stats.range + b.half_extent() {
                        plan.new_pos = Some(self.step_toward(u.pos, b.pos, u.stats.speed * dt));
                    } else if u.cooldown <= 0.0 {
                        plan.attack_building = Some(target);
                    }
                }
                _ => plan.new_order = Some(Order::Idle),
            },
            Order::Work { building } => match self.building(building) {
                Some(b) if b.nation == u.nation => {
                    // Stand at a spot around the building and get to work.
                    let angle = (u.id % 8) as f32 / 8.0 * std::f32::consts::TAU;
                    let spot = b.pos + vec2(angle.cos(), angle.sin()) * (b.half_extent() + 12.0);
                    if (spot - u.pos).length() > 4.0 {
                        let step = self.step_toward(u.pos, spot, u.stats.speed * dt);
                        plan.facing = Some((spot - u.pos).y.atan2((spot - u.pos).x));
                        plan.new_pos = Some(step);
                    }
                }
                _ => plan.new_order = Some(Order::Idle),
            },
        }
        plan
    }

    /// Straight-line steering with axis slides around impassable tiles.
    fn step_toward(&self, from: Vec2, to: Vec2, dist: f32) -> Vec2 {
        let dir = (to - from).normalize_or_zero();
        let cand = from + dir * dist;
        if self.map.passable_world(cand.x, cand.y) {
            return cand;
        }
        let x_only = from + vec2(dir.x, 0.0) * dist;
        if dir.x.abs() > 0.01 && self.map.passable_world(x_only.x, x_only.y) {
            return x_only;
        }
        let y_only = from + vec2(0.0, dir.y) * dist;
        if dir.y.abs() > 0.01 && self.map.passable_world(y_only.x, y_only.y) {
            return y_only;
        }
        from
    }

    fn nearest_enemy_unit(&self, u: &Unit, range: f32) -> Option<Id> {
        let mut best: Option<(Id, f32)> = None;
        for other in &self.units {
            if other.nation == u.nation || self.nations[other.nation].defeated {
                continue;
            }
            let d = (other.pos - u.pos).length();
            if d <= range && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((other.id, d));
            }
        }
        best.map(|(id, _)| id)
    }

    fn nearest_enemy_building(&self, u: &Unit, range: f32) -> Option<Id> {
        let mut best: Option<(Id, f32)> = None;
        for b in &self.buildings {
            if b.nation == u.nation || self.nations[b.nation].defeated {
                continue;
            }
            let d = (b.pos - u.pos).length() - b.half_extent();
            if d <= range && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((b.id, d));
            }
        }
        best.map(|(id, _)| id)
    }

    fn apply_plans(&mut self, plans: Vec<UnitPlan>, dt: f32) {
        let mut rng = ::rand::thread_rng();
        // (attacker index, damage, target) resolved after the movement pass.
        let mut unit_hits: Vec<(usize, Id)> = Vec::new();
        let mut building_hits: Vec<(usize, Id)> = Vec::new();

        for (i, plan) in plans.into_iter().enumerate() {
            let u = &mut self.units[i];
            u.cooldown = (u.cooldown - dt).max(0.0);
            if let Some(p) = plan.new_pos {
                u.pos = p;
            }
            if let Some(f) = plan.facing {
                u.facing = f;
            }
            if let Some(o) = plan.new_order {
                u.order = o;
            }
            if plan.attack_unit.is_some() || plan.attack_building.is_some() {
                u.cooldown = u.stats.cooldown;
            }
            if let Some(t) = plan.attack_unit {
                unit_hits.push((i, t));
            }
            if let Some(t) = plan.attack_building {
                building_hits.push((i, t));
            }
        }

        for (attacker_idx, target_id) in unit_hits {
            let (a_pos, a_nation, dmg, dtype, ranged) = {
                let a = &self.units[attacker_idx];
                (
                    a.pos,
                    a.nation,
                    a.stats.damage,
                    a.stats.damage_type,
                    a.stats.range > 40.0,
                )
            };
            let mut hit: Option<(Vec2, bool)> = None;
            if let Some(t) = self.units.iter_mut().find(|t| t.id == target_id) {
                let dealt = calculate_damage(dmg, dtype, t.stats.armor);
                t.hp -= dealt;
                hit = Some((t.pos, t.hp <= 0.0));
            }
            if let Some((t_pos, killed)) = hit {
                if killed {
                    self.nations[a_nation].kills += 1;
                }
                self.spawn_attack_fx(a_pos, t_pos, ranged, &mut rng);
            }
        }

        for (attacker_idx, target_id) in building_hits {
            let (a_pos, dmg, dtype, ranged) = {
                let a = &self.units[attacker_idx];
                (a.pos, a.stats.damage, a.stats.damage_type, a.stats.range > 40.0)
            };
            let a_nation = self.units[attacker_idx].nation;
            let mut hit: Option<Vec2> = None;
            if let Some(b) = self.buildings.iter_mut().find(|b| b.id == target_id) {
                b.hp -= calculate_damage(dmg, dtype, ArmorType::Building);
                b.last_attacker = Some(a_nation);
                hit = Some(b.pos);
            }
            if let Some(b_pos) = hit {
                self.spawn_attack_fx(a_pos, b_pos, ranged, &mut rng);
            }
        }

        // Damaged buildings smoulder — the Company of Heroes look.
        let mut smoke: Vec<Vec2> = Vec::new();
        for b in &mut self.buildings {
            if b.is_complete() && b.hp < b.max_hp * 0.6 {
                b.smoke_timer -= dt;
                if b.smoke_timer <= 0.0 {
                    b.smoke_timer = 0.25 + rng.gen_range(0.0..0.2);
                    smoke.push(
                        b.pos
                            + vec2(
                                rng.gen_range(-b.half_extent()..b.half_extent()),
                                rng.gen_range(-b.half_extent()..0.0),
                            ),
                    );
                }
            }
        }
        for pos in smoke {
            self.spawn_smoke(pos, &mut rng);
        }
    }

    /// Gentle O(n^2) separation so formations don't collapse into one pixel.
    fn separate_units(&mut self) {
        let n = self.units.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let delta = self.units[j].pos - self.units[i].pos;
                let min_d = self.units[i].radius() + self.units[j].radius();
                let d = delta.length();
                if d > 0.001 && d < min_d {
                    let push = delta / d * (min_d - d) * 0.5;
                    let pi = self.units[i].pos - push;
                    let pj = self.units[j].pos + push;
                    if self.map.passable_world(pi.x, pi.y) {
                        self.units[i].pos = pi;
                    }
                    if self.map.passable_world(pj.x, pj.y) {
                        self.units[j].pos = pj;
                    }
                }
            }
        }
    }

    /// Rise of Nations attrition: standing on someone else's soil hurts.
    fn attrition_tick(&mut self) {
        let mut rng = ::rand::thread_rng();
        let mut fx: Vec<Vec2> = Vec::new();
        let ages: Vec<usize> = self.nations.iter().map(|n| n.age).collect();
        let defeated: Vec<bool> = self.nations.iter().map(|n| n.defeated).collect();
        let mut kills: Vec<u32> = vec![0; self.nations.len()];

        for u in &mut self.units {
            let (tx, ty) = self.map.tile_at_world(u.pos.x, u.pos.y);
            if let Some(owner) = self.owner[(ty.max(0).min(MAP_H - 1) * MAP_W
                + tx.max(0).min(MAP_W - 1)) as usize]
            {
                let owner = owner as usize;
                if owner != u.nation && !defeated[owner] {
                    let mut dps = 1.0 + 0.5 * ages[owner] as f32;
                    if u.kind == UnitKind::Citizen {
                        dps *= 2.0;
                    }
                    u.hp -= dps;
                    if u.hp <= 0.0 {
                        kills[owner] += 1;
                    }
                    if rng.gen_bool(0.35) {
                        fx.push(u.pos);
                    }
                }
            }
        }
        for (i, k) in kills.into_iter().enumerate() {
            self.nations[i].kills += k;
        }
        for pos in fx {
            let pid = self.alloc_id();
            self.particles.push(Particle {
                id: pid,
                kind: ParticleKind::Spark,
                pos,
                vel: vec2(rng.gen_range(-6.0..6.0), -14.0),
                life: 0.5,
                max_life: 0.5,
                size: 2.0,
                color: [0.863, 0.353, 0.235, 1.000],
            });
        }
    }

    fn resolve_deaths(&mut self) {
        let mut rng = ::rand::thread_rng();

        // Dead units leave a puff.
        let mut death_fx: Vec<Vec2> = Vec::new();
        self.units.retain(|u| {
            if u.hp <= 0.0 {
                death_fx.push(u.pos);
                false
            } else {
                true
            }
        });
        for pos in death_fx {
            self.spawn_smoke(pos, &mut rng);
        }

        // Cities flip flags; everything else is rubble.
        let mut captured: Vec<(Id, usize)> = Vec::new();
        let mut destroyed_fx: Vec<Vec2> = Vec::new();
        self.buildings.retain_mut(|b| {
            if b.hp > 0.0 {
                return true;
            }
            if b.kind == BuildingKind::City {
                if let Some(conqueror) = b.last_attacker {
                    captured.push((b.id, conqueror));
                    b.nation = conqueror;
                    b.hp = b.max_hp * 0.3;
                    b.queue.clear();
                    b.queue_progress = 0.0;
                }
                true
            } else {
                destroyed_fx.push(b.pos);
                false
            }
        });
        for pos in destroyed_fx {
            for _ in 0..6 {
                self.spawn_smoke(
                    pos + vec2(rng.gen_range(-16.0..16.0), rng.gen_range(-16.0..16.0)),
                    &mut rng,
                );
            }
        }
        if !captured.is_empty() {
            self.borders_dirty = true;
            // Workers of a flipped city's nation lose nothing; citizens keep
            // orders. Capital handling happens in update_capitals.
        }
    }

    fn update_capitals(&mut self, dt: f32) {
        for i in 0..self.nations.len() {
            if self.nations[i].defeated {
                continue;
            }
            let capital_id = self.nations[i].capital;
            let holder = self.building(capital_id).map(|b| b.nation);
            match holder {
                Some(n) if n == i => self.nations[i].capital_timer = None,
                _ => {
                    // Capital in enemy hands (or gone): the countdown runs.
                    let timer = self.nations[i]
                        .capital_timer
                        .get_or_insert(CAPITAL_COUNTDOWN);
                    *timer -= dt;
                    if *timer <= 0.0 {
                        self.eliminate(i);
                    }
                }
            }
        }
    }

    /// A nation falls: its army disbands and its buildings pass to whoever
    /// holds its capital.
    fn eliminate(&mut self, nation: usize) {
        self.nations[nation].defeated = true;
        self.nations[nation].capital_timer = None;
        let conqueror = self
            .building(self.nations[nation].capital)
            .map(|b| b.nation)
            .filter(|&n| n != nation);
        self.units.retain(|u| u.nation != nation);
        for b in &mut self.buildings {
            if b.nation == nation {
                if let Some(c) = conqueror {
                    b.nation = c;
                    b.queue.clear();
                }
            }
        }
        self.borders_dirty = true;
    }

    fn check_victory(&mut self) {
        if self.winner.is_some() {
            return;
        }
        let alive: Vec<usize> = self
            .nations
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.defeated)
            .map(|(i, _)| i)
            .collect();
        if alive.len() == 1 {
            self.winner = Some(alive[0]);
        }
    }

    // -- particles ------------------------------------------------------------

    fn spawn_attack_fx(&mut self, from: Vec2, to: Vec2, ranged: bool, rng: &mut impl Rng) {
        if ranged {
            let pid = self.alloc_id();
            self.particles.push(Particle {
                id: pid,
                kind: ParticleKind::Tracer { to },
                pos: from,
                vel: Vec2::ZERO,
                life: 0.12,
                max_life: 0.12,
                size: 1.5,
                color: [1.000, 0.902, 0.627, 1.000],
            });
            let pid = self.alloc_id();
            self.particles.push(Particle {
                id: pid,
                kind: ParticleKind::Flash,
                pos: from,
                vel: Vec2::ZERO,
                life: 0.08,
                max_life: 0.08,
                size: 5.0,
                color: [1.000, 0.863, 0.471, 1.000],
            });
        }
        let pid = self.alloc_id();
        self.particles.push(Particle {
            id: pid,
            kind: ParticleKind::Blood,
            pos: to + vec2(rng.gen_range(-3.0..3.0), rng.gen_range(-3.0..3.0)),
            vel: vec2(rng.gen_range(-10.0..10.0), rng.gen_range(-18.0..-6.0)),
            life: 0.4,
            max_life: 0.4,
            size: 2.0,
            color: [0.471, 0.157, 0.118, 1.000],
        });
    }

    fn spawn_smoke(&mut self, pos: Vec2, rng: &mut impl Rng) {
        let pid = self.alloc_id();
        self.particles.push(Particle {
            id: pid,
            kind: ParticleKind::Smoke,
            pos,
            vel: vec2(rng.gen_range(-4.0..4.0), rng.gen_range(-22.0..-12.0)),
            life: 1.6,
            max_life: 1.6,
            size: rng.gen_range(4.0..9.0),
            color: [0.275, 0.267, 0.259, 0.627],
        });
    }

    fn update_particles(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.life -= dt;
            p.pos += p.vel * dt;
            if p.kind == ParticleKind::Smoke {
                p.size += 6.0 * dt;
                p.vel *= 0.98;
            }
        }
        self.particles.retain(|p| p.life > 0.0);
    }
}

/// Cost of researching the given age (1-based target index).
pub fn age_up_cost(target_age: usize) -> Cost {
    let i = target_age as f32;
    let mut c = Cost::default().food(200.0 * i).wealth(150.0 * i);
    if target_age >= 4 {
        c = c.knowledge(120.0 * (i - 3.0));
    }
    if target_age >= 6 {
        c = c.oil(80.0 * (i - 5.0));
    }
    c
}
