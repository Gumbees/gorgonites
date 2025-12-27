//! Game configuration management

use ini::Ini;
use std::path::Path;

/// Audio configuration settings
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub min_bpm: f32,
    pub max_bpm: f32,
    pub bass_volume: f32,
    pub melody_volume: f32,
    pub highs_volume: f32,
    pub bass_enabled: bool,
    pub melody_enabled: bool,
    pub highs_enabled: bool,
    pub highs_octave_offset: u8,
    pub highs_note_density: u8,
    pub evolution_min_bars: u32,
    pub evolution_max_bars: u32,
    pub mutation_chance: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 0.5,
            music_volume: 0.6,
            min_bpm: 100.0,
            max_bpm: 130.0,
            bass_volume: 0.8,
            melody_volume: 0.7,
            highs_volume: 0.5,
            bass_enabled: true,
            melody_enabled: true,
            highs_enabled: true,
            highs_octave_offset: 12,
            highs_note_density: 2,
            evolution_min_bars: 2,
            evolution_max_bars: 4,
            mutation_chance: 0.5,
        }
    }
}

/// Graphics configuration settings
#[derive(Debug, Clone)]
pub struct GraphicsConfig {
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true,
        }
    }
}

/// Gameplay configuration settings
#[derive(Debug, Clone)]
pub struct GameplayConfig {
    pub difficulty: String,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            difficulty: "normal".to_string(),
        }
    }
}

/// Master game configuration
#[derive(Debug, Clone)]
pub struct GameConfig {
    pub audio: AudioConfig,
    pub graphics: GraphicsConfig,
    pub gameplay: GameplayConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            graphics: GraphicsConfig::default(),
            gameplay: GameplayConfig::default(),
        }
    }
}

impl GameConfig {
    /// Load configuration from an INI file
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        if !path.exists() {
            tracing::info!("Config file not found at {:?}, using defaults", path);
            return Self::default();
        }

        match Ini::load_from_file(path) {
            Ok(ini) => Self::from_ini(&ini),
            Err(e) => {
                tracing::warn!("Failed to load config from {:?}: {}, using defaults", path, e);
                Self::default()
            }
        }
    }

    fn from_ini(ini: &Ini) -> Self {
        let mut config = Self::default();

        // Parse audio section
        if let Some(audio) = ini.section(Some("audio")) {
            config.audio.master_volume = parse_f32(audio.get("master_volume"), config.audio.master_volume);
            config.audio.music_volume = parse_f32(audio.get("music_volume"), config.audio.music_volume);
            config.audio.min_bpm = parse_f32(audio.get("min_bpm"), config.audio.min_bpm);
            config.audio.max_bpm = parse_f32(audio.get("max_bpm"), config.audio.max_bpm);
            config.audio.bass_volume = parse_f32(audio.get("bass_volume"), config.audio.bass_volume);
            config.audio.melody_volume = parse_f32(audio.get("melody_volume"), config.audio.melody_volume);
            config.audio.highs_volume = parse_f32(audio.get("highs_volume"), config.audio.highs_volume);
            config.audio.bass_enabled = parse_bool(audio.get("bass_enabled"), config.audio.bass_enabled);
            config.audio.melody_enabled = parse_bool(audio.get("melody_enabled"), config.audio.melody_enabled);
            config.audio.highs_enabled = parse_bool(audio.get("highs_enabled"), config.audio.highs_enabled);
            config.audio.highs_octave_offset = parse_u8(audio.get("highs_octave_offset"), config.audio.highs_octave_offset);
            config.audio.highs_note_density = parse_u8(audio.get("highs_note_density"), config.audio.highs_note_density);
            config.audio.evolution_min_bars = parse_u32(audio.get("evolution_min_bars"), config.audio.evolution_min_bars);
            config.audio.evolution_max_bars = parse_u32(audio.get("evolution_max_bars"), config.audio.evolution_max_bars);
            config.audio.mutation_chance = parse_f32(audio.get("mutation_chance"), config.audio.mutation_chance);
        }

        // Parse graphics section
        if let Some(graphics) = ini.section(Some("graphics")) {
            config.graphics.fullscreen = parse_bool(graphics.get("fullscreen"), config.graphics.fullscreen);
            config.graphics.vsync = parse_bool(graphics.get("vsync"), config.graphics.vsync);
        }

        // Parse gameplay section
        if let Some(gameplay) = ini.section(Some("gameplay")) {
            if let Some(diff) = gameplay.get("difficulty") {
                config.gameplay.difficulty = diff.to_string();
            }
        }

        config
    }

    /// Save configuration to an INI file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let mut ini = Ini::new();

        // Audio section
        ini.with_section(Some("audio"))
            .set("master_volume", self.audio.master_volume.to_string())
            .set("music_volume", self.audio.music_volume.to_string())
            .set("min_bpm", self.audio.min_bpm.to_string())
            .set("max_bpm", self.audio.max_bpm.to_string())
            .set("bass_volume", self.audio.bass_volume.to_string())
            .set("melody_volume", self.audio.melody_volume.to_string())
            .set("highs_volume", self.audio.highs_volume.to_string())
            .set("bass_enabled", self.audio.bass_enabled.to_string())
            .set("melody_enabled", self.audio.melody_enabled.to_string())
            .set("highs_enabled", self.audio.highs_enabled.to_string())
            .set("highs_octave_offset", self.audio.highs_octave_offset.to_string())
            .set("highs_note_density", self.audio.highs_note_density.to_string())
            .set("evolution_min_bars", self.audio.evolution_min_bars.to_string())
            .set("evolution_max_bars", self.audio.evolution_max_bars.to_string())
            .set("mutation_chance", self.audio.mutation_chance.to_string());

        // Graphics section
        ini.with_section(Some("graphics"))
            .set("fullscreen", self.graphics.fullscreen.to_string())
            .set("vsync", self.graphics.vsync.to_string());

        // Gameplay section
        ini.with_section(Some("gameplay"))
            .set("difficulty", &self.gameplay.difficulty);

        ini.write_to_file(path.as_ref())
            .map_err(|e| format!("Failed to save config: {}", e))
    }
}

fn parse_f32(value: Option<&str>, default: f32) -> f32 {
    value.and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn parse_u8(value: Option<&str>, default: u8) -> u8 {
    value.and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn parse_u32(value: Option<&str>, default: u32) -> u32 {
    value.and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn parse_bool(value: Option<&str>, default: bool) -> bool {
    value.map(|v| v.eq_ignore_ascii_case("true")).unwrap_or(default)
}
