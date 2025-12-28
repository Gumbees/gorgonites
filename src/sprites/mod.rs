//! AI-driven sprite generation system
//!
//! Uses Ollama to generate sprite descriptions, then renders them as pixel art.

mod generator;
mod renderer;
mod fog;

pub use generator::*;
pub use renderer::*;
pub use fog::*;
