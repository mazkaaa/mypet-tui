# MyPet TUI - Workplan

## Overview
A terminal-based virtual pet game inspired by Tamagotchi, built with Rust and Ratatui.

## Goals
- Create an interactive, real-time pet simulation in the terminal
- Learn Rust through building a complete TUI application
- Build a fun, nostalgic game experience

---

## Tech Stack

- **Language**: Rust
- **TUI Framework**: Ratatui (v0.30.0)
- **Backend**: Crossterm (default backend for Ratatui)
- **Serialization**: serde + serde_json (for save/load)
- **Async**: tokio (for time-based events)

---

## Core Features

### Phase 1: Foundation (Week 1) ✅
- [x] Project setup with Cargo
- [x] Basic Ratatui integration (terminal init, main loop, event handling)
- [x] Simple UI layout with placeholder content
- [x] Clean shutdown and terminal restoration

### Phase 2: Pet System (Week 1-2) ✅
- [x] Define Pet struct with core stats:
  - Hunger (0-100, increases over time)
  - Happiness (0-100, decreases without interaction)
  - Energy (0-100, decreases with activity)
  - Health (0-100, affected by neglect)
  - Hygiene (0-100, decreases over time)
- [x] Life stages: Egg → Baby → Child → Teen → Adult
- [x] Age system (time-based progression)
- [x] Basic ASCII art placeholders for each stage
- [x] **Egg Stage Mechanics (Enhanced):**
  - [x] **Incubation Progress**: Time-based progress bar (0-100%)
  - [x] **Warmth System**: Press [W] to warm, +10 per warm, max 100
  - [x] **Warmth Decay**: Slowly decreases over time (-3 every 5 seconds)
  - [x] **Health System**: Only visible when warmth < 30
  - [x] **Critical Mechanics**: Health drops when cold, recovers when warm
  - [x] **Game Over**: Egg dies if health reaches 0 (too cold for too long)
  - [x] **Hatching**: Auto-hatch at 100% incubation with stat bonuses based on warmth
  - [x] **Warmth Bonuses**: 
    - ≥70% warmth: Baby starts with +20 to all stats
    - 40-70% warmth: Normal starting stats
    - <40% warmth: Baby starts with -10 to all stats
  - [x] **Context-aware Messages**: 6 different status messages based on warmth level
  - [x] **EggStats Struct**: Separate struct for egg-specific data
- [x] **Baby Stage Restrictions:**
  - [x] Higher energy requirement for play (30 instead of 20)
  - [x] Less happiness gain from play (15 instead of 20)
  - [x] More energy cost from play (20 instead of 15)
  - [x] 50% more energy recovery from sleep

### Phase 3: Interactions (Week 2) ✅
- [x] Feed action (reduces hunger)
- [x] Play action (increases happiness, consumes energy)
- [x] Clean action (improves hygiene)
- [x] Sleep action (regenerates energy)
- [x] Medicine action (heals when sick)
- [x] Keyboard shortcuts for each action
- [x] **Stage-Specific Actions:**
  - [x] Egg: Only [W]arm action available
  - [x] Baby+: All actions with restrictions
  - [x] UI shows context-sensitive action bar
- [x] **Game State Management:**
  - [x] GameState enum (Playing, GameOver)
  - [x] Game over detection
  - [x] Restart functionality (press R)
  - [x] Context-sensitive action bar for game over

### Phase 4: Game Loop & Events (Week 2-3) ✅
- [x] Real-time stat decay system
- [x] Time-based aging
- [x] Random events (sickness, special moments)
- [x] Pet state machine (normal, sleeping, sick, dead)
- [x] Death mechanic when neglect is too severe
- [x] Event system with multiple event types
- [x] Event log/history panel
- [x] Context-aware events based on pet state

### Phase 5: Animation & Dynamic Pet (Week 3)
- [ ] ASCII art for each life stage
- [ ] **Dynamic Animation System:**
  - [ ] Frame-based animation engine (timer-driven frame updates)
  - [ ] Idle animations (breathing, blinking, swaying)
  - [ ] State-based animations (happy, sad, sleeping, eating, playing)
  - [ ] Transition animations between states
  - [ ] Particle effects for interactions (hearts, food, sparkles)
  - [ ] Weather/ambient effects (rain, sun, night/day cycle)
- [ ] Stats display with color-coded bars
- [ ] Event log/history panel
- [ ] Menu system with visual feedback

### Phase 6: Persistence (Week 3-4)
- [ ] Save game state to file
- [ ] Load game on startup
- [ ] Auto-save functionality
- [ ] Handle game closure gracefully

### Phase 7: Extras (Week 4+)
- [ ] Multiple pet types/species
- [ ] Mini-games for "Play" action
- [ ] Achievements system
- [ ] Sound effects (optional)
- [ ] Customization (pet name, colors)

---

## UI Layout Design

```
┌─────────────────────────────────────────────────────────┐
│  MyPet TUI - v0.1.0                           [Q]uit   │
├──────────────────────┬──────────────────────────────────┤
│                      │  Stats:                          │
│   [ASCII PET ART]    │  ┌────────────────────────────┐  │
│                      │  │ Hunger:    ████████░░  80% │  │
│   (○‿○)              │  │ Happiness: ██████░░░░  60% │  │
│                      │  │ Energy:    ████░░░░░░  40% │  │
│                      │  │ Health:    █████████░  90% │  │
│                      │  │ Hygiene:   █████░░░░░  50% │  │
│                      │  └────────────────────────────┘  │
│                      │                                  │
│                      │  Status: Happy                   │
│                      │  Age: 3 days                     │
│                      │  Stage: Child                    │
├──────────────────────┴──────────────────────────────────┤
│  Actions:                                               │
│  [F]eed  [P]lay  [C]lean  [S]leep  [M]edicine  [ESC]Menu│
├─────────────────────────────────────────────────────────┤
│  Event Log:                                             │
│  > You fed Fluffy! (+20 hunger)                         │
│  > Fluffy looks happy!                                  │
│  > Fluffy made a mess... (-10 hygiene)                  │
└─────────────────────────────────────────────────────────┘
```

---

## Architecture

### Module Structure
```
src/
├── main.rs              # Entry point, CLI args
├── app.rs               # App state and main loop
├── tui.rs               # Terminal setup and event handling
├── pet.rs               # Pet struct and logic
├── stats.rs             # Stats system
├── actions.rs           # Player actions
├── ui.rs                # UI rendering
├── animation.rs         # Animation engine and state management
├── widgets/             # Custom widgets
│   ├── mod.rs
│   ├── pet_display.rs
│   ├── animated_pet.rs  # Animated pet widget
│   ├── stat_bar.rs
│   └── event_log.rs
├── game_loop.rs         # Time-based updates
├── save.rs              # Save/load functionality
└── assets/              # ASCII art files
    ├── pets/
    │   ├── egg.txt
    │   ├── baby.txt
    │   ├── child.txt
    │   ├── teen.txt
    │   └── adult.txt
    └── animations/
        ├── idle/
        ├── actions/
        └── transitions/
```

### Key Data Structures

```rust
// Pet
struct Pet {
    name: String,
    species: Species,
    stage: LifeStage,
    age_seconds: u64,
    stats: Stats,
    state: PetState,
    birth_time: Instant,
    animation: AnimationState,  // Current animation state
}

// Stats
struct Stats {
    hunger: StatValue,      // 0-100, 0 = starving
    happiness: StatValue,   // 0-100, 0 = depressed
    energy: StatValue,      // 0-100, 0 = exhausted
    health: StatValue,      // 0-100, 0 = dead
    hygiene: StatValue,     // 0-100, 0 = filthy
}

// Animation System
struct AnimationState {
    current_animation: AnimationType,
    current_frame: usize,
    frame_timer: Instant,
    frame_duration: Duration,
    loop_count: Option<u32>,  // None = infinite loop
}

enum AnimationType {
    Idle,           // Breathing, blinking
    Happy,          // Jumping, bouncing
    Sad,            // Slouching, looking down
    Eating,         // Chomping motion
    Playing,        // Running, jumping
    Sleeping,       // Zzz animation, slow breathing
    Sick,           // Shivering, pale colors
    Evolving,       // Transformation sequence
    Transition(TransitionType),  // State transitions
}

enum TransitionType {
    WakeUp,
    FallAsleep,
    EatStart,
    EatEnd,
    PlayStart,
    PlayEnd,
    GetSick,
    Heal,
}

// Frame definition for animation
struct AnimationFrame {
    art: Vec<String>,           // Multi-line ASCII art
    color_override: Option<Color>,  // Optional color tint
    particles: Vec<Particle>,   // Visual effects
    duration: Duration,         // How long to show this frame
}

// Particle effects
struct Particle {
    position: (u16, u16),       // x, y offset from pet center
    symbol: char,               // Particle character (♥, ✦, ★, etc.)
    color: Color,
    lifetime: Duration,
    velocity: (i8, i8),         // Movement direction
}
```

// Life Stages
enum LifeStage {
    Egg,
    Baby,
    Child,
    Teen,
    Adult,
}

// Pet States
enum PetState {
    Normal,
    Sleeping,
    Sick,
    Dead,
}

// Actions
enum Action {
    Feed,
    Play,
    Clean,
    Sleep,
    Medicine,
    Quit,
}
```

---

## Animation System Design

### Frame-Based Animation Engine
- **Timer-driven updates**: Check animation timer every render cycle
- **Frame interpolation**: Smooth transitions between animation states
- **Priority system**: Interruptible animations (action > transition > idle)
- **Loop management**: Configurable loops per animation type

### Animation Types

#### 1. Idle Animations (Infinite Loop)
- **Breathing**: Subtle vertical scale changes (1-2 lines)
- **Blinking**: Periodic eye closure (random intervals 2-5 seconds)
- **Swaying**: Gentle horizontal movement (1-2 character shifts)
- **Variation**: Different idle poses based on happiness level

#### 2. State-Based Animations
- **Happy**: Bouncing/jumping, sparkles, hearts floating up
- **Sad**: Slouched posture, slow breathing, occasional sigh particle
- **Sleeping**: "Zzz" particles floating up, dimmed colors
- **Sick**: Shivering vibration, pale color tint, sweat drops
- **Eating**: Chomping motion, food particles, "nom nom" text
- **Playing**: Rapid movement, excitement particles

#### 3. Transition Animations (One-shot)
- **Wake Up**: Eyes opening slowly, stretching
- **Fall Asleep**: Eyes closing, slumping down
- **Evolution**: Flash effect, transformation sequence, sparkle burst
- **Feed/Clean/Play**: Quick action-specific animation

### Visual Effects

#### Particles
- **Food**: 🍖, 🍎, 🐟 floating toward mouth
- **Hearts**: ♥ floating up when happy
- **Stars**: ✦, ★ for special moments/evolution
- **Zzz**: Z particles for sleeping
- **Sweat/Clean**: 💧, ✨ for cleaning/healing

#### Color Effects
- **Flash**: Bright white flash for evolution/level up
- **Pulse**: Gentle brightness breathing for idle
- **Tint**: Color overlays based on state (green=sick, red=angry)
- **Dim**: Reduce brightness when sleeping or low energy

### Animation Assets Structure
```
assets/animations/
├── idle/
│   ├── happy/
│   │   ├── frame_1.txt
│   │   ├── frame_2.txt
│   │   └── frame_3.txt
│   ├── neutral/
│   └── sad/
├── actions/
│   ├── eat/
│   ├── play/
│   ├── clean/
│   └── sleep/
├── transitions/
│   ├── wake_up/
│   ├── fall_asleep/
│   └── evolve/
└── particles/
    ├── hearts.json
    ├── food.json
    └── sparkles.json
```

### Technical Implementation
- Use `tokio::time::interval` for animation frame updates
- Separate render thread for smooth 30 FPS animation
- Frame buffer to prevent tearing
- Lazy-load animation assets to reduce startup time

---

## Development Milestones

### Milestone 1: Hello Pet
- Terminal opens with a static pet display
- Can quit with 'q'

### Milestone 2: Living Pet
- Stats that change over time
- Can interact with pet
- Pet reacts to interactions

### Milestone 3: Complete Lifecycle
- Pet ages through stages
- Death mechanics
- New game after death

### Milestone 4: Persistence
- Save and load working
- Progress is preserved

---

## Open Questions

1. **Time Scale**: Real-time (1 sec = 1 sec) or accelerated (1 sec = 1 minute)?
2. **ASCII Art Style**: Minimalist (◕‿◕) or detailed multi-line art?
3. **Difficulty**: How fast should stats decay?
4. **Notifications**: Show desktop notifications when pet needs attention?
5. **Animation Frame Rate**: Target 10 FPS (subtle) or 30 FPS (smooth) for animations?
6. **Animation Complexity**: Pre-rendered frames or procedural animations (moving parts)?
7. **Particles**: Simple ASCII characters or use Unicode emojis/symbols?
8. **Screen Space**: Fixed pet size or resize with terminal?

---

## Resources

- [Ratatui Documentation](https://ratatui.rs/)
- [Ratatui GitHub](https://github.com/ratatui/ratatui)
- [Crossterm Documentation](https://docs.rs/crossterm/latest/crossterm/)
- ASCII Art inspiration: Text to ASCII Art Generators

---

## Success Criteria

- [ ] Can start game and see a pet
- [ ] Can interact with pet using keyboard
- [ ] Pet stats change over time
- [ ] Pet evolves through life stages
- [ ] **Pet displays idle animations (breathing, blinking)**
- [ ] **Pet shows state-based animations (happy, sad, sleeping)**
- [ ] **Visual feedback on interactions (particles, effects)**
- [ ] Can save and load game
- [ ] Clean exit without breaking terminal

---

*Created: February 2026*
*Last Updated: February 2026*
