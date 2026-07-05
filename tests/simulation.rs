//! Headless simulation tests for the Rise of Nations-style world.
//!
//! These drive `World::update` directly (no window, no rendering) to prove
//! the core loops hold together: borders, economy, AI build-up, attrition,
//! city capture, and the capital countdown.

use bevy::math::vec2;

use gorgonites::game::{
    age_up_cost, BuildingKind, Order, QueueItem, UnitKind, World, MAP_H, MAP_W, TILE,
};

fn run(world: &mut World, seconds: f32) {
    let dt = 0.05;
    let steps = (seconds / dt) as usize;
    for _ in 0..steps {
        world.update(dt);
    }
}

#[test]
fn world_initializes_with_two_nations_and_borders() {
    let world = World::new();
    assert_eq!(world.nations.len(), 2);
    assert_eq!(
        world
            .buildings
            .iter()
            .filter(|b| b.kind == BuildingKind::City)
            .count(),
        2
    );
    // Each capital projects national territory.
    let owned = (0..MAP_H)
        .flat_map(|y| (0..MAP_W).map(move |x| (x, y)))
        .filter(|&(x, y)| world.tile_owner(x, y).is_some())
        .count();
    assert!(owned > 100, "borders should cover territory, got {owned}");
    // Both nations start under the population cap with starting citizens.
    for nation in &world.nations {
        assert!(nation.pop > 0 && nation.pop <= nation.pop_cap);
    }
}

#[test]
fn economy_and_ai_progress_over_five_minutes() {
    let mut world = World::new();
    let start_food = world.nations[0].stockpile.get(gorgonites::systems::rts::Resource::Food);
    run(&mut world, 300.0);

    assert!(world.winner.is_none(), "nobody should win in 5 idle minutes");
    // City trickle alone should grow the player's food.
    let food = world.nations[0].stockpile.get(gorgonites::systems::rts::Resource::Food);
    assert!(food > start_food, "city trickle should accumulate food");
    // The AI nation must have expanded beyond its capital and its 4 citizens.
    let ai_buildings = world.buildings.iter().filter(|b| b.nation == 1).count();
    assert!(ai_buildings > 1, "AI should construct buildings, has {ai_buildings}");
    let ai_citizens = world.count_units(1, UnitKind::Citizen);
    assert!(ai_citizens > 4, "AI should train citizens, has {ai_citizens}");
}

#[test]
fn attrition_bleeds_units_on_enemy_soil() {
    let mut world = World::new();
    // Drop a player soldier in the middle of enemy territory.
    let enemy_capital = world.building(world.nations[1].capital).unwrap().pos;
    let id = world.spawn_unit(0, UnitKind::Infantry, enemy_capital + vec2(TILE * 4.0, 0.0));
    let start_hp = world.unit(id).unwrap().hp;
    run(&mut world, 10.0);
    let unit = world.unit(id);
    match unit {
        Some(u) => assert!(
            u.hp < start_hp,
            "attrition should bleed the intruder ({} -> {})",
            start_hp,
            u.hp
        ),
        None => {} // died to attrition + defenders: also proof it works
    }
}

#[test]
fn cities_flip_flags_instead_of_dying() {
    let mut world = World::new();
    let enemy_capital = world.nations[1].capital;
    {
        let b = world
            .buildings
            .iter_mut()
            .find(|b| b.id == enemy_capital)
            .unwrap();
        b.hp = 0.0;
        b.last_attacker = Some(0);
    }
    world.update(0.05);
    let capital = world.building(enemy_capital).expect("city must survive capture");
    assert_eq!(capital.nation, 0, "city should change flags to the attacker");
    assert!(capital.hp > 0.0, "captured city keeps some hit points");
    // The dispossessed nation is now on the capital countdown.
    assert!(world.nations[1].capital_timer.is_some());

    // Held long enough, the nation falls and the battle ends.
    run(&mut world, 65.0);
    assert!(world.nations[1].defeated, "losing the capital ends the nation");
    assert_eq!(world.winner, Some(0));
}

#[test]
fn age_advancement_is_paid_and_researched_at_the_city() {
    let mut world = World::new();
    let capital = world.nations[0].capital;

    // Can't afford age 2 from starting resources.
    assert!(world.try_enqueue(capital, QueueItem::AgeUp).is_err());

    // Fund it and research it.
    let cost = age_up_cost(1);
    world.nations[0].stockpile.refund(&cost);
    world.try_enqueue(capital, QueueItem::AgeUp).expect("funded age-up enqueues");
    run(&mut world, 30.0);
    assert_eq!(world.nations[0].age, 1, "age research should complete");
}

#[test]
fn ramping_costs_and_pop_cap_enforced() {
    let mut world = World::new();
    let capital = world.nations[0].capital;

    // Ramping: with citizens alive, the next citizen costs more than base.
    let base = gorgonites::game::unit_base_cost(UnitKind::Citizen);
    let ramped = gorgonites::game::unit_ramped_cost(
        UnitKind::Citizen,
        world.count_units(0, UnitKind::Citizen),
    );
    assert!(ramped.0[0] > base.0[0], "unit costs must ramp");

    // Pop cap: force pop to the cap and training must refuse.
    for _ in 0..200 {
        if world.nations[0].pop >= world.nations[0].pop_cap {
            break;
        }
        let pos = world.building(capital).unwrap().pos + vec2(TILE * 3.0, 0.0);
        world.spawn_unit(0, UnitKind::Citizen, pos);
        world.update(0.01);
    }
    world.nations[0].stockpile.refund(&gorgonites::systems::rts::Cost::default().food(100000.0));
    let err = world.try_enqueue(capital, QueueItem::Unit(UnitKind::Citizen));
    assert!(err.is_err(), "training past the population cap must fail");
}

#[test]
fn workers_generate_capped_income() {
    let mut world = World::new();
    // Build a farm next to the player capital and staff it.
    let capital_pos = world.building(world.nations[0].capital).unwrap().pos;
    let tile = (
        (capital_pos.x / TILE) as i32 + 3,
        (capital_pos.y / TILE) as i32,
    );
    // Find a legal nearby spot.
    let mut spot = None;
    'outer: for dy in -6..=6 {
        for dx in -6..=6 {
            let t = (tile.0 + dx, tile.1 + dy);
            if world.can_place(0, BuildingKind::Farm, t).is_ok() {
                spot = Some(t);
                break 'outer;
            }
        }
    }
    let farm = world.place_building(0, BuildingKind::Farm, spot.expect("farm spot"), true);
    let citizens: Vec<_> = world
        .units
        .iter()
        .filter(|u| u.nation == 0 && u.kind == UnitKind::Citizen)
        .map(|u| u.id)
        .collect();
    for id in citizens {
        if let Some(u) = world.units.iter_mut().find(|u| u.id == id) {
            u.order = Order::Work { building: farm };
        }
    }
    run(&mut world, 30.0);
    let food_income = world.nations[0].income[0];
    assert!(
        food_income > 1.0,
        "staffed farm should produce food income, got {food_income}"
    );
}
