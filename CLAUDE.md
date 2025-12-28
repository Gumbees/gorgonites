# Gorgonites

A historical strategy game spanning all of time - from Stone Age to far future, reality to fiction.

## Tech Stack

- **Language:** Rust
- **Game Framework:** macroquad
- **AI Integration:** Ollama (local LLM for sprite generation)
- **Audio:** Custom procedural music generation

## Project Structure

```
src/
├── main.rs              # Entry point
├── game/                # Core game state and loop
│   ├── mod.rs           # Game struct, update/render loop
│   ├── state.rs         # GameState enum
│   └── era.rs           # Historical eras
├── sprites/             # AI-driven sprite generation
│   ├── generator.rs     # Ollama integration, sprite descriptions
│   ├── renderer.rs      # Pixel art rendering
│   └── fog.rs           # Fog particle system (disabled)
├── audio/               # Procedural music
├── ai/                  # Ollama client
├── ui/                  # Menu, HUD, dialogue
└── config/              # Configuration loading
```

## Building & Running

```bash
cargo build
cargo run
```

## Configuration

Edit `config.ini` for:
- Audio settings (volume, BPM, track toggles)
- Ollama settings (host, port, model, timeout)
- Graphics settings

Current Ollama model: `qwen2.5:7b` with 120s timeout

## Sprite Generation

The game uses Ollama to generate unique warrior descriptions, then renders them as pixel art.

**Archetypes:** Tiny, Light, Medium, Heavy, Giant, Hulk
**Helmet Styles:** 22 types (none, hood, mask, helm, crested, horned, samurai, mandalorian, cyber, wizard, skull, etc.)
**Weapons:** 40+ types spanning all eras (club, sword, katana, musket, lightsaber, plasma gun, etc.)

## Known Issues

- #1: AI sprite colors and armor variety need improvement
- Fog system disabled (too many draw calls crash display driver)

## Development Notes

- Sprites render at 48x48 pixels for maximum detail
- 5-level color shading: highlight, light, mid, dark, shadow
- Mouse movement triggers AI sprite generation on main menu
- Sequential generation (one at a time to avoid overwhelming Ollama)
