//! AI integration module
//!
//! Handles communication with LLMs for narrative generation.

mod client;
mod ollama;
mod prompts;

pub use client::*;
pub use ollama::*;
pub use prompts::*;
