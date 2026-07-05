//! Units and buildings: the concrete pieces on the battlefield.
//!
//! Rise of Nations rules encoded here:
//! - unit lines that re-skin and scale through the eight ages
//! - ramping costs (each additional unit of a line costs more)
//! - cities are captured, never destroyed; everything else burns
//! - economic buildings hold citizen worker slots that generate resource *rates*

use macroquad::prelude::*;

use crate::systems::rts::{ArmorType, Cost, DamageType, Resource};

pub type Id = u32;

// ---------------------------------------------------------------------------
// Units
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Citizen,
    Infantry,
    Ranged,
    Cavalry,
    Siege,
}

impl UnitKind {
    pub const MILITARY: [UnitKind; 4] = [
        UnitKind::Infantry,
        UnitKind::Ranged,
        UnitKind::Cavalry,
        UnitKind::Siege,
    ];

    pub fn is_military(&self) -> bool {
        !matches!(self, UnitKind::Citizen)
    }
}

/// Era-flavoured display name for a unit line (Rise of Nations upgrades the
/// same line through the ages: Clubman -> Legionnaire -> ... -> Exo Trooper).
pub fn unit_name(kind: UnitKind, age: usize) -> &'static str {
    const INFANTRY: [&str; 8] = [
        "Clubman", "Spearman", "Legionnaire", "Man-at-Arms", "Musketeer", "Rifleman",
        "Assault Infantry", "Exo Trooper",
    ];
    const RANGED: [&str; 8] = [
        "Slinger", "Archer", "Composite Archer", "Crossbowman", "Arquebusier", "Skirmisher",
        "Sniper", "Rail Gunner",
    ];
    const CAVALRY: [&str; 8] = [
        "Raider", "Chariot", "Cataphract", "Knight", "Cuirassier", "Lancer", "Tank",
        "Hover Tank",
    ];
    const SIEGE: [&str; 8] = [
        "Battering Ram", "Catapult", "Ballista", "Trebuchet", "Bombard", "Field Cannon",
        "Howitzer", "Plasma Mortar",
    ];
    let age = age.min(7);
    match kind {
        UnitKind::Citizen => "Citizen",
        UnitKind::Infantry => INFANTRY[age],
        UnitKind::Ranged => RANGED[age],
        UnitKind::Cavalry => CAVALRY[age],
        UnitKind::Siege => SIEGE[age],
    }
}

/// Snapshot of a unit's combat statistics at creation time.
#[derive(Debug, Clone, Copy)]
pub struct UnitStats {
    pub max_hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub range: f32,
    pub cooldown: f32,
    pub damage_type: DamageType,
    pub armor: ArmorType,
    pub pop: i32,
    pub train_time: f32,
    pub aggro_range: f32,
}

/// Stats for a unit line at a given age. Later ages hit harder, live longer,
/// and move a touch faster — the same line, upgraded.
pub fn unit_stats(kind: UnitKind, age: usize) -> UnitStats {
    let a = age.min(7) as f32;
    let scale = 1.0 + 0.30 * a;
    let base = match kind {
        UnitKind::Citizen => UnitStats {
            max_hp: 40.0,
            speed: 70.0,
            damage: 3.0,
            range: 18.0,
            cooldown: 1.5,
            damage_type: DamageType::Melee,
            armor: ArmorType::Unarmored,
            pop: 1,
            train_time: 9.0,
            aggro_range: 0.0,
        },
        UnitKind::Infantry => UnitStats {
            max_hp: 90.0,
            speed: 78.0,
            damage: 9.0,
            range: 20.0,
            cooldown: 1.2,
            damage_type: DamageType::Melee,
            armor: ArmorType::Medium,
            pop: 1,
            train_time: 11.0,
            aggro_range: 165.0,
        },
        UnitKind::Ranged => UnitStats {
            max_hp: 55.0,
            speed: 74.0,
            damage: 7.0,
            range: 150.0,
            cooldown: 1.6,
            damage_type: DamageType::Ranged,
            armor: ArmorType::Light,
            pop: 1,
            train_time: 11.0,
            aggro_range: 185.0,
        },
        UnitKind::Cavalry => UnitStats {
            max_hp: 135.0,
            speed: 122.0,
            damage: 12.0,
            range: 22.0,
            cooldown: 1.3,
            damage_type: DamageType::Melee,
            armor: ArmorType::Medium,
            pop: 2,
            train_time: 15.0,
            aggro_range: 175.0,
        },
        UnitKind::Siege => UnitStats {
            max_hp: 70.0,
            speed: 46.0,
            damage: 32.0,
            range: 210.0,
            cooldown: 3.6,
            damage_type: DamageType::Siege,
            armor: ArmorType::Unarmored,
            pop: 3,
            train_time: 22.0,
            aggro_range: 200.0,
        },
    };
    UnitStats {
        max_hp: base.max_hp * scale,
        damage: base.damage * scale,
        speed: base.speed + 2.0 * a,
        ..base
    }
}

/// Base price of a unit line before ramping.
pub fn unit_base_cost(kind: UnitKind) -> Cost {
    match kind {
        UnitKind::Citizen => Cost::default().food(50.0),
        UnitKind::Infantry => Cost::default().food(60.0).timber(20.0),
        UnitKind::Ranged => Cost::default().food(40.0).timber(50.0),
        UnitKind::Cavalry => Cost::default().food(80.0).wealth(40.0),
        UnitKind::Siege => Cost::default().timber(80.0).metal(60.0).wealth(40.0),
    }
}

/// Rise of Nations ramping: every living unit of the line makes the next one
/// pricier. Keeps armies mixed and spam expensive.
pub fn unit_ramped_cost(kind: UnitKind, existing_of_kind: usize) -> Cost {
    unit_base_cost(kind).scaled(1.0 + 0.12 * existing_of_kind as f32)
}

/// What a unit is currently doing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Order {
    Idle,
    /// Move to a point; `aggro` units engage enemies met along the way.
    Move { dest: Vec2, aggro: bool },
    AttackUnit(Id),
    AttackBuilding(Id),
    /// Citizen assigned as a worker at an economic building.
    Work { building: Id },
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub id: Id,
    pub nation: usize,
    pub kind: UnitKind,
    pub pos: Vec2,
    pub facing: f32,
    pub hp: f32,
    pub stats: UnitStats,
    pub order: Order,
    pub cooldown: f32,
}

impl Unit {
    pub fn radius(&self) -> f32 {
        match self.kind {
            UnitKind::Citizen => 6.0,
            UnitKind::Infantry | UnitKind::Ranged => 7.0,
            UnitKind::Cavalry => 9.0,
            UnitKind::Siege => 10.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Buildings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingKind {
    City,
    Farm,
    LumberCamp,
    Mine,
    Market,
    University,
    Barracks,
    OilWell,
}

impl BuildingKind {
    /// Buildings a citizen can place, in UI order.
    pub const BUILDABLE: [BuildingKind; 8] = [
        BuildingKind::Farm,
        BuildingKind::LumberCamp,
        BuildingKind::Mine,
        BuildingKind::Market,
        BuildingKind::University,
        BuildingKind::Barracks,
        BuildingKind::OilWell,
        BuildingKind::City,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            BuildingKind::City => "City",
            BuildingKind::Farm => "Farm",
            BuildingKind::LumberCamp => "Lumber Camp",
            BuildingKind::Mine => "Mine",
            BuildingKind::Market => "Market",
            BuildingKind::University => "University",
            BuildingKind::Barracks => "Barracks",
            BuildingKind::OilWell => "Oil Well",
        }
    }

    pub fn cost(&self) -> Cost {
        match self {
            BuildingKind::City => Cost::default().food(150.0).timber(300.0).wealth(100.0),
            BuildingKind::Farm => Cost::default().timber(80.0),
            BuildingKind::LumberCamp => Cost::default().timber(60.0),
            BuildingKind::Mine => Cost::default().timber(100.0),
            BuildingKind::Market => Cost::default().timber(120.0),
            BuildingKind::University => Cost::default().timber(140.0).wealth(60.0),
            BuildingKind::Barracks => Cost::default().timber(110.0),
            BuildingKind::OilWell => Cost::default().timber(150.0).metal(100.0),
        }
    }

    pub fn max_hp(&self) -> f32 {
        match self {
            BuildingKind::City => 1500.0,
            BuildingKind::Farm => 200.0,
            BuildingKind::LumberCamp => 250.0,
            BuildingKind::Mine => 300.0,
            BuildingKind::Market => 300.0,
            BuildingKind::University => 350.0,
            BuildingKind::Barracks => 500.0,
            BuildingKind::OilWell => 350.0,
        }
    }

    pub fn build_time(&self) -> f32 {
        match self {
            BuildingKind::City => 40.0,
            BuildingKind::Farm => 12.0,
            BuildingKind::LumberCamp => 10.0,
            BuildingKind::Mine => 14.0,
            BuildingKind::Market => 14.0,
            BuildingKind::University => 16.0,
            BuildingKind::Barracks => 15.0,
            BuildingKind::OilWell => 18.0,
        }
    }

    /// Citizen worker slots and what each worker produces per second.
    pub fn output(&self) -> Option<(Resource, f32, usize)> {
        match self {
            BuildingKind::Farm => Some((Resource::Food, 0.9, 5)),
            BuildingKind::LumberCamp => Some((Resource::Timber, 0.8, 4)),
            BuildingKind::Mine => Some((Resource::Metal, 0.6, 4)),
            BuildingKind::Market => Some((Resource::Wealth, 0.7, 3)),
            BuildingKind::University => Some((Resource::Knowledge, 0.5, 3)),
            BuildingKind::OilWell => Some((Resource::Oil, 0.8, 3)),
            _ => None,
        }
    }

    /// Footprint side length in tiles.
    pub fn footprint(&self) -> i32 {
        match self {
            BuildingKind::City => 3,
            _ => 2,
        }
    }

    /// Minimum age (0-based) required to construct this.
    pub fn min_age(&self) -> usize {
        match self {
            BuildingKind::University => 1,
            BuildingKind::OilWell => 5,
            _ => 0,
        }
    }
}

/// Something a production building is working on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueueItem {
    Unit(UnitKind),
    /// Researching the advance to the next age (queued at a city).
    AgeUp,
}

#[derive(Debug, Clone)]
pub struct Building {
    pub id: Id,
    pub nation: usize,
    pub kind: BuildingKind,
    /// Top-left tile of the footprint.
    pub tile: (i32, i32),
    /// World-space centre.
    pub pos: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    /// 0.0..1.0; below 1.0 the building is under construction.
    pub construction: f32,
    pub queue: Vec<QueueItem>,
    pub queue_progress: f32,
    /// Nation index of the last attacker (city capture credit).
    pub last_attacker: Option<usize>,
    /// Damaged-building smoke emission timer.
    pub smoke_timer: f32,
}

impl Building {
    pub fn half_extent(&self) -> f32 {
        self.kind.footprint() as f32 * super::mapgen::TILE / 2.0
    }

    pub fn is_complete(&self) -> bool {
        self.construction >= 1.0
    }
}
