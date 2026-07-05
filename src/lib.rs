//! Gorgonites - An AI-driven alternate history strategy game
//!
//! This crate provides the core game logic for Gorgonites: a Rise of
//! Nations-style battle simulation (`game`), the supporting strategy and
//! narrative systems (`systems`, `ai`), and the Bevy 3D frontend
//! (`frontend`). The simulation never touches the engine, so it runs
//! headlessly in tests and the renderer can evolve independently.

pub mod ai;
pub mod assets;
pub mod frontend;
pub mod game;
pub mod systems;
