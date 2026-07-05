//! The opposing nation's brain.
//!
//! A deliberately readable build-order AI: staff the economy, expand the
//! building stock, climb the ages, and throw periodic attack waves at the
//! player's capital — enough to make borders, attrition, and capture matter.

use macroquad::prelude::*;

use super::entities::*;
use super::mapgen::TILE;
use super::world::World;

pub struct AiState {
    think_timer: f32,
    /// Seconds until the next attack wave is allowed.
    wave_timer: f32,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            think_timer: 0.0,
            wave_timer: 160.0,
        }
    }
}

pub fn ai_tick(world: &mut World, nation: usize, state: &mut AiState, dt: f32) {
    state.wave_timer -= dt;
    state.think_timer -= dt;
    if state.think_timer > 0.0 {
        return;
    }
    state.think_timer = 2.0;

    assign_idle_citizens(world, nation);
    manage_economy(world, nation);
    manage_military(world, nation);
    consider_age_up(world, nation);

    if state.wave_timer <= 0.0 {
        if launch_attack_wave(world, nation) {
            state.wave_timer = 110.0;
        } else {
            state.wave_timer = 20.0;
        }
    }
}

fn assign_idle_citizens(world: &mut World, nation: usize) {
    // Collect buildings with free worker slots.
    let mut open: Vec<(Id, usize)> = world
        .buildings
        .iter()
        .filter(|b| b.nation == nation && b.is_complete())
        .filter_map(|b| {
            b.kind.output().map(|(_, _, slots)| {
                let free = slots.saturating_sub(world.workers_at(b.id));
                (b.id, free)
            })
        })
        .filter(|(_, free)| *free > 0)
        .collect();

    let idle: Vec<Id> = world
        .units
        .iter()
        .filter(|u| u.nation == nation && u.kind == UnitKind::Citizen && u.order == Order::Idle)
        .map(|u| u.id)
        .collect();

    for citizen in idle {
        let Some(slot) = open.iter_mut().find(|(_, free)| *free > 0) else {
            break;
        };
        let building = slot.0;
        slot.1 -= 1;
        if let Some(u) = world.units.iter_mut().find(|u| u.id == citizen) {
            u.order = Order::Work { building };
        }
    }
}

fn manage_economy(world: &mut World, nation: usize) {
    let age = world.nations[nation].age;
    let citizens = world.count_units(nation, UnitKind::Citizen);
    let target_citizens = 8 + 2 * age;

    // Keep the citizen line running at the capital.
    if citizens < target_citizens {
        let capital = world.nations[nation].capital;
        if world
            .building(capital)
            .map_or(false, |b| b.queue.len() < 2)
        {
            let _ = world.try_enqueue(capital, QueueItem::Unit(UnitKind::Citizen));
        }
    }

    // Build order: the economy RoN wants — food, timber, wealth, metal,
    // knowledge, army production, then oil once industrial.
    let desired: [(BuildingKind, usize); 8] = [
        (BuildingKind::Farm, 2 + age / 2),
        (BuildingKind::LumberCamp, 2),
        (BuildingKind::Market, 1),
        (BuildingKind::Mine, 1 + age / 3),
        (BuildingKind::Barracks, 1 + age / 3),
        (BuildingKind::University, if age >= 1 { 1 } else { 0 }),
        (BuildingKind::Farm, 3 + age / 2),
        (BuildingKind::OilWell, if age >= 5 { 1 } else { 0 }),
    ];

    for (kind, want) in desired {
        if want == 0 || world.count_buildings(nation, kind) >= want {
            continue;
        }
        let cost = kind.cost();
        if !world.nations[nation].stockpile.can_afford(&cost) {
            continue;
        }
        if let Some(tile) = find_spot(world, nation, kind) {
            if world.nations[nation].stockpile.pay(&cost) {
                world.place_building(nation, kind, tile, false);
            }
        }
        break; // one construction start per think tick
    }
}

fn manage_military(world: &mut World, nation: usize) {
    let age = world.nations[nation].age;
    let army: usize = UnitKind::MILITARY
        .iter()
        .map(|&k| world.count_units(nation, k))
        .sum();
    let target = 6 + 4 * age;
    if army >= target {
        return;
    }

    let barracks: Vec<Id> = world
        .buildings
        .iter()
        .filter(|b| {
            b.nation == nation && b.kind == BuildingKind::Barracks && b.is_complete()
                && b.queue.len() < 2
        })
        .map(|b| b.id)
        .collect();

    for (i, b) in barracks.into_iter().enumerate() {
        let kind = match (army + i) % 4 {
            0 | 1 => UnitKind::Infantry,
            2 => UnitKind::Ranged,
            _ if age >= 2 => UnitKind::Cavalry,
            _ => UnitKind::Ranged,
        };
        let _ = world.try_enqueue(b, QueueItem::Unit(kind));
    }
}

fn consider_age_up(world: &mut World, nation: usize) {
    let age = world.nations[nation].age;
    if age >= 7 {
        return;
    }
    let cost = super::world::age_up_cost(age + 1);
    // Only commit when comfortably affordable so the economy keeps breathing.
    if world.nations[nation].stockpile.can_afford(&cost.scaled(1.3)) {
        let capital = world.nations[nation].capital;
        let _ = world.try_enqueue(capital, QueueItem::AgeUp);
    }
}

fn launch_attack_wave(world: &mut World, nation: usize) -> bool {
    let soldiers: Vec<Id> = world
        .units
        .iter()
        .filter(|u| u.nation == nation && u.kind.is_military())
        .map(|u| u.id)
        .collect();
    if soldiers.len() < 6 {
        return false;
    }

    // March on the weakest enemy capital.
    let target = world
        .nations
        .iter()
        .enumerate()
        .filter(|(i, n)| *i != nation && !n.defeated)
        .filter_map(|(_, n)| world.building(n.capital).map(|b| b.pos))
        .next();
    let Some(target_pos) = target else {
        return false;
    };

    for (i, id) in soldiers.iter().enumerate() {
        if let Some(u) = world.units.iter_mut().find(|u| u.id == *id) {
            let offset = vec2(
                ((i % 4) as f32 - 1.5) * 24.0,
                ((i / 4) as f32) * 24.0,
            );
            u.order = Order::Move {
                dest: target_pos + offset,
                aggro: true,
            };
        }
    }
    true
}

/// Spiral outward from the capital looking for a legal placement tile.
fn find_spot(world: &World, nation: usize, kind: BuildingKind) -> Option<(i32, i32)> {
    let capital = world.building(world.nations[nation].capital)?;
    let (cx, cy) = (
        (capital.pos.x / TILE) as i32,
        (capital.pos.y / TILE) as i32,
    );
    for radius in 3i32..16 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue; // ring only
                }
                let tile = (cx + dx, cy + dy);
                if world.can_place(nation, kind, tile).is_ok() {
                    return Some(tile);
                }
            }
        }
    }
    None
}
