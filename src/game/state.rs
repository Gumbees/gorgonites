//! Game state definitions

/// The current state of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// Main menu screen
    MainMenu,

    /// Era/scenario selection
    EraSelect,

    /// Game is actively running
    Playing,

    /// Game is paused
    Paused,

    /// Narrative event is being displayed (player must choose)
    NarrativeChoice,

    /// Game over screen
    GameOver,

    /// Loading screen
    Loading,
}
