//! Gorgonites - An AI-driven alternate history strategy game
//!
//! This crate provides the core game logic for Gorgonites, a strategy game
//! that blends RTS mechanics, D&D-style narrative, and grand strategy.

pub mod game;
pub mod ecs;
pub mod systems;
pub mod ai;
pub mod rendering;
pub mod ui;
pub mod assets;

pub use game::Game;
