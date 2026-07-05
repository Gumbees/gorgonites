# Gorgonites

**An AI-driven alternate history strategy game**

*"History is written by the victors. You decide who wins."*

## Vision

Gorgonites is a 3D strategy game that blends:
- **Real-Time Strategy** - Command units, manage resources, wage war
- **D&D-style Narrative** - AI dungeon master drives story and forces meaningful choices
- **Axis & Allies Strategic Depth** - Grand strategy with historical weight

The AI narrator is not a helper—it's the game. It generates scenarios, presents dilemmas, narrates consequences, and watches your civilization diverge from our timeline into something entirely new.

## Core Concept

### The Timeline Engine

Every game begins at a chosen era:
- **Stone Age** - Tribal survival, fire, first tools
- **Bronze Age** - Early empires, chariots, written law
- **Iron Age** - Classical civilizations, philosophy, legions
- **Medieval** - Feudalism, castles, faith and steel
- **Renaissance** - Gunpowder, exploration, printing press
- **Industrial** - Factories, railways, nationalism
- **Modern** - Total war, nuclear age, information era

Your choices compound. Discover gunpowder early? The Medieval period looks very different. Fail to develop writing? Your empire crumbles under its own complexity.

### AI as Narrator

The AI doesn't just respond—it *drives*:
- Generates historical scenarios with branching consequences
- Creates characters, factions, and crises
- Forces impossible choices ("Save the library or the granary?")
- Narrates the ripple effects across generations
- Tracks how far your timeline has diverged from reality

### Divergence Score

A core metric tracking how different your world is from ours:
- **0-20%**: Familiar history with minor changes
- **21-50%**: Recognizable but altered (Rome never fell, China discovered America)
- **51-80%**: Radically different (No monotheism, Bronze Age never ended)
- **81-100%**: Alien world (Unrecognizable civilization paths)

High divergence unlocks stranger scenarios. Low divergence lets you "fix" history or optimize known paths.

## Gameplay Pillars

### 1. Strategic Command (RTS Layer — Rise of Nations model)

The battle layer plays like **Rise of Nations**, presented with a grounded,
muted **Company of Heroes**-style art direction (earthy palettes, soft
shadows, tracers and drifting smoke — not saturated cartoon RTS):

- **National borders** — every city projects territory; borders grow with new
  cities and each age. You can only construct inside your own borders.
- **Attrition** — enemy units bleed hit points every second they stand on your
  soil, scaling with your age. Invasions have a running cost.
- **Six-resource economy** — Food, Timber, Metal, Wealth, Knowledge, Oil.
  Citizens staff farms, lumber camps, mines, markets, universities, and oil
  wells to generate continuous *rates*, clamped per resource by a
  **commerce cap** that rises with each age.
- **Ramping costs** — every additional unit of a line costs more, so armies
  stay mixed and spam stays expensive.
- **City capture, not destruction** — reduce a city to rubble and it changes
  flags. Lose your capital and a 60-second countdown starts; retake it or
  your nation falls. Take the enemy capital and hold it to win.
- **Eight ages** — advance from the Stone Age to the Divergent timeline at a
  city. Every age re-skins and upgrades each unit line (Clubman →
  Legionnaire → Musketeer → Exo Trooper), extends borders, raises the
  commerce cap, and sharpens attrition.
- An **AI opponent nation** runs the same rules: it staffs its economy,
  expands, climbs the ages, and launches attack waves at your capital.

> Art note: the game runs on a real 3D engine (Bevy + PBR) with real CC0
> assets. The battlefield is a normal-mapped, PBR-textured terrain mesh
> (ambientCG grass) under a Poly Haven sky HDRI, with water, distance fog,
> bloom, a warm sun with cascaded shadows, scattered low-poly forests and
> boulders, national borders projected on the ground, and tracer/smoke
> effects. Buildings are PBR-textured (stone/wood/metal); units are real
> glTF character models (KayKit, CC0 — knight/rogue/barbarian) on
> nation-coloured team discs, loaded via a kind→model registry with a
> primitive fallback. The frontend is fully isolated from the simulation, so
> more/animated models drop in without touching gameplay — model animation
> and roofed building models are the remaining steps toward full Company of
> Heroes fidelity. Bundled assets and their sources are listed in
> [`assets/CREDITS.md`](./assets/CREDITS.md).

### 2. Narrative Choices (D&D Layer)
- AI presents scenarios between/during battles
- Choices affect unit morale, tech paths, faction relations
- Characters can become heroes or villains
- No "right" answers—only consequences

### 3. Grand Strategy (Axis & Allies Layer)
- Multiple factions competing across the map
- Diplomacy, trade, espionage
- Long-term resource and territory control
- Era transitions as major game phases

## Technical Stack

- **Engine**: Bevy (Rust) - data-driven ECS with a 3D PBR renderer
- **AI Integration**: LLM-powered narrator (local + API options)
- **Architecture**: Entity-Component-System for game objects
- **Serialization**: Serde for save states and timeline data

## Project Structure

The simulation (`game/`) is engine-agnostic — it depends on math types only,
never on Bevy — so it runs headlessly in tests and the 3D frontend
(`frontend/`) can evolve independently.

```
gorgonites/
├── src/
│   ├── main.rs              # Entry point (calls frontend::run)
│   ├── lib.rs               # Library root
│   ├── game/                # Engine-agnostic battle simulation
│   │   ├── world.rs         # Borders, attrition, capture, economy, combat
│   │   ├── entities.rs      # Unit/building stats, era lines, ramping costs
│   │   ├── mapgen.rs        # Noise terrain + elevation + oil deposits
│   │   ├── ai_nation.rs     # Opponent nation AI
│   │   └── era.rs           # The eight ages
│   ├── frontend/            # Bevy 3D frontend (isolated from the sim)
│   │   ├── sim.rs           # Sim-as-Resource, fixed-timestep tick
│   │   ├── scene.rs         # Terrain mesh, sun, water, fog, coord bridge
│   │   ├── camera.rs        # RTS camera (pan/zoom/rotate)
│   │   ├── input.rs         # Ray-pick selection, orders, placement
│   │   ├── sync.rs          # Sim → 3D entities + gizmo overlays
│   │   ├── hud.rs           # Menu, resource bar, action bar, end screen
│   │   └── capture.rs       # Headless screenshot aid (env-gated)
│   ├── systems/             # Strategy/narrative/timeline model
│   ├── ai/                  # LLM integration (narrator)
│   └── assets/              # Asset manifest types
├── assets/                  # Game assets (models, audio, data)
├── docs/                    # Design documents
└── tests/                   # Headless simulation tests
```

## Development Roadmap

### Phase 1: Foundation
- [x] Project scaffold
- [x] Bevy app + fixed-timestep game loop
- [x] ECS via Bevy (sim mirrored into entities each frame)
- [x] 3D rendering: lit terrain mesh, water, fog, shadows, border overlays

### Phase 2: RTS Core (Rise of Nations rules)
- [x] Unit spawning, drag-select, right-click orders
- [x] Movement (steering; full pathfinding later) and combat
- [x] Six-resource commerce-capped economy with citizen workers
- [x] National borders + build-inside-borders + attrition
- [x] City capture, capital countdown, victory conditions
- [x] Eight-age advancement with era-scaled unit lines
- [x] AI opponent nation (economy, expansion, attack waves)
- [ ] A* pathfinding and formations
- [ ] Naval and air units

### Phase 3: Narrative Engine
- [ ] AI integration (API calls)
- [ ] Scenario generation
- [ ] Choice presentation UI
- [ ] Consequence tracking

### Phase 4: Timeline System
- [ ] Era definitions and transitions
- [ ] Divergence scoring
- [ ] Tech tree with branching
- [ ] Historical event triggers

### Phase 5: Grand Strategy
- [ ] Multiple factions
- [ ] Diplomacy system
- [ ] Territory control
- [ ] Victory conditions

## Name Origin

The Gorgonites are the "enemy" toys from *Small Soldiers* (1998)—peaceful creatures forced into war by their programming. In this game, you're both the Gorgonites and the Commando Elite. The AI shapes the conflict, but you choose who you become.

---

## Getting Started

```bash
# Clone the repository
git clone https://github.com/Gumbees/gorgonites.git
cd gorgonites

# Run the game
cargo run

# Run tests (headless battle-sim tests — no display needed)
cargo test
```

On Linux, Bevy needs the usual windowing/graphics dev packages (X11 +
Vulkan). On Debian/Ubuntu: `sudo apt install libx11-dev libxkbcommon-x11-0
libwayland-dev libvulkan1 mesa-vulkan-drivers` (add `wayland` back to the
Bevy features in `Cargo.toml` if you run a Wayland session).

### Controls

| Input | Action |
|---|---|
| Left-drag / left-click | Select units / building |
| Right-click | Move, attack, or assign citizens to a work site |
| WASD / arrow keys | Pan camera |
| Q / E | Rotate camera; scroll to zoom |
| Action-bar buttons | Train units, place buildings, advance the age |
| ESC | Cancel building placement |

## License

TBD

## Contributing

This is currently a personal project. Design discussions welcome via issues.
