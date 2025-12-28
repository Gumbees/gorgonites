//! Sprite description generator using Ollama

use crate::ai::{OllamaClient, OllamaError};
use crate::config::OllamaConfig;
use rand::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// A generated sprite description
#[derive(Debug, Clone)]
pub struct SpriteDescription {
    /// Primary color (RGB)
    pub primary_color: [u8; 3],
    /// Secondary color (RGB)
    pub secondary_color: [u8; 3],
    /// Accent color (RGB)
    pub accent_color: [u8; 3],
    /// Shape type
    pub shape: SpriteShape,
    /// Size in pixels (width, height)
    pub size: (u8, u8),
    /// Pattern type
    pub pattern: SpritePattern,
    /// Creature name/type
    pub name: String,
    /// Warrior archetype (affects body shape and gear)
    pub archetype: WarriorArchetype,
    /// Helmet style
    pub helmet: HelmetStyle,
    /// Weapon type
    pub weapon: WeaponType,
    /// Has shield
    pub has_shield: bool,
    /// Has cape/cloak
    pub has_cape: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WarriorArchetype {
    Light,    // Scouts, archers, rogues - slim
    Medium,   // Infantry, soldiers - normal
    Heavy,    // Knights, hoplites - broad/stocky
    Giant,    // Berserkers, orcs - tall and muscular
    Tiny,     // Goblins, halflings - small
    Hulk,     // Demons, mechs - massive
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HelmetStyle {
    // Primitive
    None,           // Bare head / tribal / alien
    Hood,           // Cloth hood / cloak
    Mask,           // Tribal mask / ninja mask
    // Ancient
    Cap,            // Simple cap/hat
    Helm,           // Full helmet
    Crested,        // Roman/Spartan style with crest
    Horned,         // Viking / demon horns
    Crown,          // Commander/king/emperor
    Turban,         // Middle Eastern / mystic
    // Eastern
    Samurai,        // Japanese kabuto
    Conical,        // Asian conical hat
    // Colonial/Modern
    Tricorn,        // Pirate / colonial era
    Beret,          // Modern military
    Gasmask,        // WWI/dystopian
    // Sci-Fi
    Visor,          // Futuristic visor / Cyclops
    Mandalorian,    // Full face sci-fi helmet
    Pilot,          // Space pilot / fighter pilot
    Cyber,          // Cyberpunk implants
    // Fantasy
    Wizard,         // Pointed wizard hat
    Halo,           // Angelic ring
    Skull,          // Skeleton / death knight
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponType {
    // Primitive
    Club,
    Stone,
    Bone,
    // Bronze/Iron Age
    Sword,
    Spear,
    Axe,
    Bow,
    Mace,
    Trident,
    Dagger,
    Scythe,
    // Medieval
    Halberd,
    Crossbow,
    Flail,
    Warhammer,
    // Far East
    Katana,
    Naginata,
    Nunchaku,
    Shuriken,
    // Renaissance+
    Pike,
    Musket,
    Saber,
    Rapier,
    // Modern
    Rifle,
    Pistol,
    Grenade,
    Flamethrower,
    // Sci-Fi
    LaserRifle,
    PlasmaGun,
    Lightsaber,
    RailGun,
    BeamSword,
    // Fantasy/Magic
    Staff,
    Wand,
    Orb,
    Torch,
    Scimitar,
    HolyBlade,
    DemonSword,
    // Exotic
    Whip,
    Chakram,
    Claws,
    Gauntlet,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpriteShape {
    Blob,
    Humanoid,
    Quadruped,
    Flying,
    Serpent,
    Geometric,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpritePattern {
    Solid,
    Striped,
    Spotted,
    Gradient,
    Checkered,
}

impl Default for SpriteDescription {
    fn default() -> Self {
        Self {
            primary_color: [40, 80, 160],    // Royal blue armor
            secondary_color: [210, 180, 140], // Skin tone
            accent_color: [255, 215, 0],      // Gold trim
            shape: SpriteShape::Humanoid,
            size: (48, 48),  // Large for maximum detail
            pattern: SpritePattern::Solid,
            name: "Unknown Warrior".to_string(),
            archetype: WarriorArchetype::Medium,
            helmet: HelmetStyle::Helm,
            weapon: WeaponType::Sword,
            has_shield: true,
            has_cape: false,
        }
    }
}

/// Message types for async sprite generation
pub enum SpriteMessage {
    CheckAvailability,
    AvailabilityResult(bool),
    Generate { x: f32, y: f32 },
    Result(SpriteDescription, f32, f32),
    Error(String, f32, f32),
}

/// Manages async sprite generation via Ollama
pub struct SpriteGenerator {
    sender: Sender<SpriteMessage>,
    receiver: Receiver<SpriteMessage>,
    is_generating: bool,
    ollama_available: bool,
    checking_availability: bool,
}

impl SpriteGenerator {
    /// Create a new sprite generator
    pub fn new(config: &OllamaConfig) -> Self {
        let (tx_to_worker, rx_from_main) = channel::<SpriteMessage>();
        let (tx_to_main, rx_from_worker) = channel::<SpriteMessage>();

        let config = config.clone();

        // Spawn worker thread for Ollama requests
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            while let Ok(msg) = rx_from_main.recv() {
                match msg {
                    SpriteMessage::CheckAvailability => {
                        let available = rt.block_on(check_ollama_available(&config));
                        let _ = tx_to_main.send(SpriteMessage::AvailabilityResult(available));
                    }
                    SpriteMessage::Generate { x, y } => {
                        let result = rt.block_on(generate_sprite_async(&config, x, y));
                        match result {
                            Ok(desc) => {
                                let _ = tx_to_main.send(SpriteMessage::Result(desc, x, y));
                            }
                            Err(e) => {
                                let _ = tx_to_main.send(SpriteMessage::Error(e.to_string(), x, y));
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        let mut gen = Self {
            sender: tx_to_worker,
            receiver: rx_from_worker,
            is_generating: false,
            ollama_available: false,
            checking_availability: false,
        };

        // Start checking availability immediately
        gen.check_availability();

        gen
    }

    /// Check Ollama availability
    fn check_availability(&mut self) {
        if !self.checking_availability {
            self.checking_availability = true;
            let _ = self.sender.send(SpriteMessage::CheckAvailability);
        }
    }

    /// Request a new sprite at the given position
    /// Returns true if request was accepted, false if busy or unavailable
    pub fn request_sprite(&mut self, x: f32, y: f32) -> bool {
        // Only generate one at a time, and only if Ollama is available
        if !self.is_generating && self.ollama_available {
            let _ = self.sender.send(SpriteMessage::Generate { x, y });
            self.is_generating = true;
            true
        } else {
            false
        }
    }

    /// Poll for completed sprites and status updates (non-blocking)
    pub fn poll(&mut self) -> Option<(SpriteDescription, f32, f32)> {
        // Process all pending messages
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                SpriteMessage::AvailabilityResult(available) => {
                    self.checking_availability = false;
                    if available && !self.ollama_available {
                        tracing::info!("Ollama is now available for sprite generation");
                    }
                    self.ollama_available = available;
                }
                SpriteMessage::Result(desc, x, y) => {
                    self.is_generating = false;
                    return Some((desc, x, y));
                }
                SpriteMessage::Error(e, _x, _y) => {
                    self.is_generating = false;
                    tracing::warn!("Sprite generation error: {}", e);
                    // Mark as unavailable and start rechecking
                    self.ollama_available = false;
                    self.check_availability();
                    // Don't return a sprite - just skip this one
                }
                _ => {}
            }
        }
        None
    }

    /// Periodically recheck availability if not available
    pub fn update(&mut self) {
        if !self.ollama_available && !self.checking_availability && !self.is_generating {
            self.check_availability();
        }
    }

    /// Check if currently generating a sprite
    pub fn is_busy(&self) -> bool {
        self.is_generating
    }

    /// Check if Ollama is available
    pub fn is_available(&self) -> bool {
        self.ollama_available
    }

    /// Check if currently checking availability
    pub fn is_checking(&self) -> bool {
        self.checking_availability
    }
}

/// Check if Ollama is available and has the required model
async fn check_ollama_available(config: &OllamaConfig) -> bool {
    match OllamaClient::new(config) {
        Ok(client) => {
            // Check if service is up
            match client.health_check().await {
                Ok(true) => {
                    // Check if model is available by listing models
                    match client.list_models().await {
                        Ok(models) => {
                            let has_model = models.iter().any(|m| m.starts_with(&config.model));
                            if !has_model {
                                tracing::debug!("Model '{}' not found. Available: {:?}", config.model, models);
                            }
                            has_model
                        }
                        Err(_) => false,
                    }
                }
                _ => false,
            }
        }
        Err(_) => false,
    }
}

/// Generate a sprite description using Ollama
async fn generate_sprite_async(
    config: &OllamaConfig,
    x: f32,
    y: f32,
) -> Result<SpriteDescription, OllamaError> {
    tracing::debug!("Starting sprite generation at ({}, {})", x, y);

    let client = OllamaClient::new(config)?;

    let prompt = r#"Create a UNIQUE warrior/fighter from ANY time period, reality, or fiction!

Examples: Viking Berserker, Jedi Knight, Roman Centurion, Space Marine, Ninja Assassin,
Orc Warlord, Aztec Eagle Warrior, Cyberpunk Mercenary, Medieval Paladin, Spartan Hoplite,
Samurai Ronin, WW1 Stormtrooper, Pirate Captain, Mandalorian Bounty Hunter, Dark Elf Ranger,
Egyptian Pharaoh Guard, Mongol Horse Archer, Steampunk Automaton, Demon Knight, Angel Warrior

BE CREATIVE! Mix eras, add fantasy/sci-fi, invent something new!

USE BOLD, VIBRANT COLORS - EACH WARRIOR SHOULD BE DIFFERENT!
Pick from the FULL spectrum - don't repeat the same colors:
Royal Blue [30,60,180], Emerald Green [30,180,60], Imperial Purple [120,40,160],
Neon Cyan [0,220,220], Gold [220,180,40], Crimson Red [180,30,30],
Ice Blue [100,180,255], Toxic Green [80,255,80], Hot Pink [255,60,150],
Electric Blue [40,100,255], Burning Orange [255,100,30], Midnight Black [20,20,30],
Forest Green [40,100,40], Silver [180,180,200], Blood Red [150,20,20]

Reply with ONLY:
NAME: [Be creative and specific!]
PRIMARY: [R,G,B] (armor/body - PICK A UNIQUE COLOR for this warrior!)
SECONDARY: [R,G,B] (skin tone or contrasting cloth)
ACCENT: [R,G,B] (weapon glow/trim - make it POP!)
BUILD: [tiny/light/medium/heavy/giant/hulk]
HELMET: [none/hood/mask/cap/helm/crested/horned/crown/turban/samurai/conical/tricorn/beret/gasmask/visor/mandalorian/pilot/cyber/wizard/halo/skull]
WEAPON: [club/stone/bone/sword/spear/axe/bow/mace/trident/dagger/scythe/halberd/crossbow/flail/warhammer/katana/naginata/nunchaku/shuriken/pike/musket/saber/rapier/rifle/pistol/grenade/flamethrower/laserrifle/plasmagun/lightsaber/railgun/beamsword/staff/wand/orb/torch/scimitar/holyblade/demonsword/whip/chakram/claws/gauntlet]
SHIELD: [yes/no]
CAPE: [yes/no]"#;

    tracing::debug!("Sending request to Ollama...");
    let response = client.generate(prompt, Some("Reply with the exact format requested. Be brief.")).await?;
    tracing::debug!("Got response from Ollama: {} chars", response.len());

    // Parse the response
    parse_sprite_response(&response)
}

/// Parse Ollama's response into a SpriteDescription
fn parse_sprite_response(response: &str) -> Result<SpriteDescription, OllamaError> {
    let mut desc = SpriteDescription::default();
    let mut rng = rand::thread_rng();

    for line in response.lines() {
        let line = line.trim();

        if let Some(name) = line.strip_prefix("NAME:") {
            desc.name = name.trim().trim_matches('"').to_string();
        } else if let Some(color) = line.strip_prefix("PRIMARY:") {
            if let Some(rgb) = parse_rgb(color) {
                desc.primary_color = rgb;
            }
        } else if let Some(color) = line.strip_prefix("SECONDARY:") {
            if let Some(rgb) = parse_rgb(color) {
                desc.secondary_color = rgb;
            }
        } else if let Some(color) = line.strip_prefix("ACCENT:") {
            if let Some(rgb) = parse_rgb(color) {
                desc.accent_color = rgb;
            }
        } else if let Some(build) = line.strip_prefix("BUILD:") {
            desc.archetype = match build.trim().to_lowercase().as_str() {
                "tiny" => WarriorArchetype::Tiny,
                "light" => WarriorArchetype::Light,
                "medium" => WarriorArchetype::Medium,
                "heavy" => WarriorArchetype::Heavy,
                "giant" => WarriorArchetype::Giant,
                "hulk" => WarriorArchetype::Hulk,
                _ => WarriorArchetype::Medium,
            };
        } else if let Some(helmet) = line.strip_prefix("HELMET:") {
            desc.helmet = match helmet.trim().to_lowercase().as_str() {
                "none" => HelmetStyle::None,
                "hood" => HelmetStyle::Hood,
                "mask" => HelmetStyle::Mask,
                "cap" => HelmetStyle::Cap,
                "helm" | "helmet" => HelmetStyle::Helm,
                "crested" => HelmetStyle::Crested,
                "horned" | "horns" => HelmetStyle::Horned,
                "crown" => HelmetStyle::Crown,
                "turban" => HelmetStyle::Turban,
                "samurai" | "kabuto" => HelmetStyle::Samurai,
                "conical" => HelmetStyle::Conical,
                "tricorn" => HelmetStyle::Tricorn,
                "beret" => HelmetStyle::Beret,
                "gasmask" | "gas mask" => HelmetStyle::Gasmask,
                "visor" => HelmetStyle::Visor,
                "mandalorian" | "mando" => HelmetStyle::Mandalorian,
                "pilot" => HelmetStyle::Pilot,
                "cyber" | "cyberpunk" => HelmetStyle::Cyber,
                "wizard" | "mage" => HelmetStyle::Wizard,
                "halo" | "angelic" => HelmetStyle::Halo,
                "skull" | "skeleton" => HelmetStyle::Skull,
                _ => HelmetStyle::Helm,
            };
        } else if let Some(weapon) = line.strip_prefix("WEAPON:") {
            desc.weapon = match weapon.trim().to_lowercase().as_str() {
                // Primitive
                "club" => WeaponType::Club,
                "stone" => WeaponType::Stone,
                "bone" => WeaponType::Bone,
                // Ancient
                "sword" => WeaponType::Sword,
                "spear" => WeaponType::Spear,
                "axe" => WeaponType::Axe,
                "bow" => WeaponType::Bow,
                "mace" => WeaponType::Mace,
                "trident" => WeaponType::Trident,
                "dagger" | "knife" => WeaponType::Dagger,
                "scythe" => WeaponType::Scythe,
                // Medieval
                "halberd" => WeaponType::Halberd,
                "crossbow" => WeaponType::Crossbow,
                "flail" => WeaponType::Flail,
                "warhammer" | "hammer" => WeaponType::Warhammer,
                // Eastern
                "katana" => WeaponType::Katana,
                "naginata" => WeaponType::Naginata,
                "nunchaku" | "nunchucks" => WeaponType::Nunchaku,
                "shuriken" | "throwing star" => WeaponType::Shuriken,
                // Renaissance
                "pike" => WeaponType::Pike,
                "musket" => WeaponType::Musket,
                "saber" | "sabre" => WeaponType::Saber,
                "rapier" => WeaponType::Rapier,
                // Modern
                "rifle" => WeaponType::Rifle,
                "pistol" | "gun" => WeaponType::Pistol,
                "grenade" => WeaponType::Grenade,
                "flamethrower" => WeaponType::Flamethrower,
                // Sci-Fi
                "laserrifle" | "laser rifle" | "laser" => WeaponType::LaserRifle,
                "plasmagun" | "plasma gun" | "plasma" => WeaponType::PlasmaGun,
                "lightsaber" | "light saber" | "energy sword" => WeaponType::Lightsaber,
                "railgun" | "rail gun" => WeaponType::RailGun,
                "beamsword" | "beam sword" => WeaponType::BeamSword,
                // Fantasy
                "staff" => WeaponType::Staff,
                "wand" => WeaponType::Wand,
                "orb" => WeaponType::Orb,
                "torch" => WeaponType::Torch,
                "scimitar" => WeaponType::Scimitar,
                "holyblade" | "holy blade" | "holy sword" => WeaponType::HolyBlade,
                "demonsword" | "demon sword" | "cursed blade" => WeaponType::DemonSword,
                // Exotic
                "whip" => WeaponType::Whip,
                "chakram" => WeaponType::Chakram,
                "claws" => WeaponType::Claws,
                "gauntlet" | "power fist" => WeaponType::Gauntlet,
                _ => WeaponType::Sword,
            };
        } else if let Some(shield) = line.strip_prefix("SHIELD:") {
            desc.has_shield = shield.trim().to_lowercase() == "yes";
        } else if let Some(cape) = line.strip_prefix("CAPE:") {
            desc.has_cape = cape.trim().to_lowercase() == "yes";
        } else if let Some(shape) = line.strip_prefix("SHAPE:") {
            desc.shape = match shape.trim().to_lowercase().as_str() {
                "blob" => SpriteShape::Blob,
                "humanoid" => SpriteShape::Humanoid,
                "quadruped" => SpriteShape::Quadruped,
                "flying" => SpriteShape::Flying,
                "serpent" => SpriteShape::Serpent,
                "geometric" => SpriteShape::Geometric,
                _ => SpriteShape::Humanoid,
            };
        } else if let Some(pattern) = line.strip_prefix("PATTERN:") {
            desc.pattern = match pattern.trim().to_lowercase().as_str() {
                "solid" => SpritePattern::Solid,
                "striped" => SpritePattern::Striped,
                "spotted" => SpritePattern::Spotted,
                "gradient" => SpritePattern::Gradient,
                "checkered" => SpritePattern::Checkered,
                _ => SpritePattern::Solid,
            };
        } else if let Some(size) = line.strip_prefix("SIZE:") {
            if let Some((w, h)) = parse_size(size) {
                desc.size = (w, h);
            }
        }
    }

    // If name is still default, generate a random one
    if desc.name == "Unknown Warrior" {
        desc.name = generate_random_name(&mut rng);
    }

    // Force humanoid shape for warriors
    desc.shape = SpriteShape::Humanoid;

    Ok(desc)
}

/// Parse RGB from string like "[255, 128, 64]" or "255,128,64"
fn parse_rgb(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_matches(|c| c == '[' || c == ']');
    let parts: Vec<&str> = s.split(',').collect();

    if parts.len() >= 3 {
        let r = parts[0].trim().parse::<u8>().ok()?;
        let g = parts[1].trim().parse::<u8>().ok()?;
        let b = parts[2].trim().parse::<u8>().ok()?;
        Some([r, g, b])
    } else {
        None
    }
}

/// Parse size from string like "[32, 32]" or "32,32"
fn parse_size(s: &str) -> Option<(u8, u8)> {
    let s = s.trim().trim_matches(|c| c == '[' || c == ']');
    let parts: Vec<&str> = s.split(',').collect();

    if parts.len() >= 2 {
        let w = parts[0].trim().parse::<u8>().ok()?.clamp(16, 64);
        let h = parts[1].trim().parse::<u8>().ok()?.clamp(16, 64);
        Some((w, h))
    } else {
        None
    }
}

/// Generate a random warrior name
fn generate_random_name(rng: &mut impl Rng) -> String {
    let adjectives = ["Iron", "Bronze", "Stone", "War", "Battle", "Shield", "Spear", "Sword", "Axe", "Bow"];
    let units = ["Warrior", "Soldier", "Guard", "Archer", "Spearman", "Swordsman", "Knight", "Legionary", "Tribesman", "Berserker"];

    format!(
        "{} {}",
        adjectives.choose(rng).unwrap(),
        units.choose(rng).unwrap()
    )
}
