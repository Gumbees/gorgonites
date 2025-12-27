//! Audio system for procedural music generation

mod synth;
mod melody;
mod music;

pub use music::ProceduralMusic;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use rodio::{OutputStream, OutputStreamHandle, Sink};

/// Manages audio playback for the game
pub struct AudioManager {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    music_sink: Option<Sink>,
    music_playing: Arc<AtomicBool>,
    seed: u64,
}

impl AudioManager {
    /// Create a new audio manager with a random seed based on current time
    pub fn new() -> Result<Self, AudioError> {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);

        Self::with_seed(seed)
    }

    /// Create a new audio manager with a specific seed (for reproducible music)
    pub fn with_seed(seed: u64) -> Result<Self, AudioError> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::OutputStreamError(e.to_string()))?;

        tracing::info!("Audio system initialized with seed: {}", seed);

        Ok(Self {
            _stream: stream,
            _stream_handle: stream_handle,
            music_sink: None,
            music_playing: Arc::new(AtomicBool::new(false)),
            seed,
        })
    }

    /// Start playing procedural menu music
    pub fn play_menu_music(&mut self) {
        if self.music_playing.load(Ordering::Relaxed) {
            return; // Already playing
        }

        match Sink::try_new(&self._stream_handle) {
            Ok(sink) => {
                let music = ProceduralMusic::new(self.seed);
                sink.set_volume(0.5);
                sink.append(music);

                self.music_sink = Some(sink);
                self.music_playing.store(true, Ordering::Relaxed);

                tracing::info!("Menu music started");
            }
            Err(e) => {
                tracing::warn!("Failed to create audio sink: {}", e);
            }
        }
    }

    /// Stop the currently playing music
    pub fn stop_music(&mut self) {
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
            self.music_playing.store(false, Ordering::Relaxed);
            tracing::info!("Music stopped");
        }
    }

    /// Pause the music (can be resumed)
    pub fn pause_music(&mut self) {
        if let Some(ref sink) = self.music_sink {
            sink.pause();
        }
    }

    /// Resume paused music
    pub fn resume_music(&mut self) {
        if let Some(ref sink) = self.music_sink {
            sink.play();
        }
    }

    /// Check if music is currently playing
    pub fn is_music_playing(&self) -> bool {
        self.music_playing.load(Ordering::Relaxed)
    }

    /// Set music volume (0.0 - 1.0)
    pub fn set_music_volume(&mut self, volume: f32) {
        if let Some(ref sink) = self.music_sink {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::error!("Failed to initialize audio: {:?}", e);
            panic!("Audio initialization failed: {:?}", e);
        })
    }
}

/// Audio system errors
#[derive(Debug)]
pub enum AudioError {
    OutputStreamError(String),
    PlaybackError(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::OutputStreamError(msg) => write!(f, "Output stream error: {}", msg),
            AudioError::PlaybackError(msg) => write!(f, "Playback error: {}", msg),
        }
    }
}

impl std::error::Error for AudioError {}
