//! The simulation as a Bevy resource.
//!
//! `crate::game::World` is the engine-agnostic sim. Here it becomes a
//! `Resource`, ticked once per frame while in `AppState::Playing`, with a
//! fixed-substep accumulator so gameplay speed is frame-rate independent.

use bevy::prelude::*;

use crate::game::World as SimWorld;

use super::AppState;

/// Wraps the sim so Bevy systems can borrow it.
#[derive(Resource)]
pub struct Sim {
    pub world: SimWorld,
    accumulator: f32,
}

impl Default for Sim {
    fn default() -> Self {
        Self {
            world: SimWorld::new(),
            accumulator: 0.0,
        }
    }
}

impl Sim {
    /// Player nation is always index 0.
    pub const PLAYER: usize = 0;

    pub fn reset(&mut self) {
        // Fresh procedural map for every new scenario.
        self.world = SimWorld::new_random();
        self.accumulator = 0.0;
    }
}

/// Fixed sim timestep (seconds). The sim was tuned at ~20 Hz.
const SIM_DT: f32 = 1.0 / 30.0;

/// Regenerating the world (fresh map) must happen before any `OnEnter(Playing)`
/// system that reads `Sim` to build meshes — the terrain mesh and unit/building
/// assets. Those systems order themselves `.after(WorldSetup)`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldSetup;

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Sim>()
            .add_systems(OnEnter(AppState::Playing), reset_sim.in_set(WorldSetup))
            .add_systems(Update, tick_sim.run_if(in_state(AppState::Playing)));
    }
}

fn reset_sim(mut sim: ResMut<Sim>) {
    sim.reset();
}

fn tick_sim(
    time: Res<Time>,
    mut sim: ResMut<Sim>,
    mut next: ResMut<NextState<AppState>>,
) {
    // Clamp the frame delta so a stall doesn't spiral the accumulator.
    sim.accumulator += time.delta_secs().min(0.1);
    let mut steps = 0;
    while sim.accumulator >= SIM_DT && steps < 6 {
        sim.world.update(SIM_DT);
        sim.accumulator -= SIM_DT;
        steps += 1;
    }

    if sim.world.winner.is_some() || sim.world.nations[Sim::PLAYER].defeated {
        next.set(AppState::GameOver);
    }
}
