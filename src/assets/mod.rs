//! Asset loading and management

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Asset manifest for tracking game assets
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetManifest {
    /// Sprite assets
    pub sprites: HashMap<String, SpriteAsset>,

    /// Audio assets
    pub audio: HashMap<String, AudioAsset>,

    /// Data files
    pub data: HashMap<String, DataAsset>,
}

/// A sprite/texture asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteAsset {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub frames: Option<u32>,
}

/// An audio asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAsset {
    pub path: String,
    pub volume: f32,
    pub looping: bool,
}

/// A data file asset (JSON, TOML, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAsset {
    pub path: String,
    pub asset_type: DataAssetType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataAssetType {
    UnitDefinitions,
    BuildingDefinitions,
    TechTree,
    EventTemplates,
    Localization,
    Config,
}

impl AssetManifest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load manifest from TOML file
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))
    }

    /// Save manifest to TOML file
    pub fn save(&self, path: &str) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write manifest: {}", e))
    }
}

/// Paths to asset directories
pub mod paths {
    pub const SPRITES: &str = "assets/sprites";
    pub const AUDIO: &str = "assets/audio";
    pub const DATA: &str = "assets/data";
    pub const MANIFEST: &str = "assets/manifest.toml";
}

/// Create default asset manifest
pub fn create_default_manifest() -> AssetManifest {
    let mut manifest = AssetManifest::new();

    // Placeholder sprites
    manifest.sprites.insert(
        "unit_warrior".to_string(),
        SpriteAsset {
            path: "sprites/units/warrior.png".to_string(),
            width: 32,
            height: 32,
            frames: Some(4),
        },
    );

    manifest.sprites.insert(
        "building_hut".to_string(),
        SpriteAsset {
            path: "sprites/buildings/hut.png".to_string(),
            width: 64,
            height: 64,
            frames: None,
        },
    );

    // Data files
    manifest.data.insert(
        "units".to_string(),
        DataAsset {
            path: "data/units.toml".to_string(),
            asset_type: DataAssetType::UnitDefinitions,
        },
    );

    manifest.data.insert(
        "tech_tree".to_string(),
        DataAsset {
            path: "data/tech_tree.toml".to_string(),
            asset_type: DataAssetType::TechTree,
        },
    );

    manifest
}
