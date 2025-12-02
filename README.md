# Gorgonites

**An AI-driven alternate history strategy game**

*"History is written by the victors. You decide who wins."*

## Vision

Gorgonites is a 2D strategy game that blends:
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

### 1. Strategic Command (RTS Layer)
- Control units on a 2D battlefield
- Resource gathering and management
- Base building appropriate to era
- Tech tree that branches based on choices, not just research

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

- **Engine**: Macroquad (Rust) - lightweight, cross-platform 2D
- **AI Integration**: LLM-powered narrator (local + API options)
- **Architecture**: Entity-Component-System for game objects
- **Serialization**: Serde for save states and timeline data

## Project Structure

```
gorgonites/
├── src/
│   ├── main.rs              # Entry point and game loop
│   ├── lib.rs               # Library root
│   ├── game/                # Core game state and loop
│   ├── ecs/                 # Entity-Component-System
│   ├── systems/             # Game systems
│   │   ├── rts/             # Unit control, resources, combat
│   │   ├── narrative/       # AI narrator, choices, dialogue
│   │   ├── timeline/        # Era progression, divergence
│   │   └── strategy/        # Diplomacy, factions, grand map
│   ├── ai/                  # LLM integration
│   ├── rendering/           # Macroquad rendering
│   ├── ui/                  # Menus, HUD, dialogue boxes
│   └── assets/              # Asset loading and management
├── assets/                  # Game assets (sprites, audio, data)
├── docs/                    # Design documents
└── tests/                   # Integration tests
```

## Development Roadmap

### Phase 1: Foundation
- [x] Project scaffold
- [ ] Basic macroquad game loop
- [ ] Simple ECS implementation
- [ ] Placeholder rendering

### Phase 2: RTS Core
- [ ] Unit spawning and selection
- [ ] Movement and pathfinding
- [ ] Basic combat
- [ ] Resource system

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

# Run tests
cargo test
```

## License

TBD

## Contributing

This is currently a personal project. Design discussions welcome via issues.
