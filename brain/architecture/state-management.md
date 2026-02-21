# State Management

## Overview

This document describes how application state is structured, mutated, and maintained in MyPet TUI.

## State Architecture: Hierarchical Ownership

```
App (root)
├── Pet (lifecycle and identity)
│   ├── Stats (mutable values 0-100)
│   ├── LifeStage (enum progression)
│   ├── PetState (current behavior state)
│   └── AnimationState (current visual state)
├── GameState (running/paused)
├── EventLog (history)
└── Config (settings)
```

## Core State Structures

### Pet State

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    // Identity
    pub name: String,
    pub species: Species,
    pub birth_time: DateTime<Utc>,
    
    // Core state
    pub stage: LifeStage,
    pub state: PetState,
    pub stats: Stats,
    
    // Visual state
    pub animation: AnimationState,
    
    // Metadata
    pub last_interaction: DateTime<Utc>,
    pub total_play_time: Duration,
}
```

### Stats System

Stats are bounded values (0-100) with automatic clamping:

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    pub hunger: StatValue,      // 0 = full, 100 = starving
    pub happiness: StatValue,   // 0 = depressed, 100 = ecstatic
    pub energy: StatValue,      // 0 = exhausted, 100 = energetic
    pub health: StatValue,      // 0 = dead, 100 = perfect
    pub hygiene: StatValue,     // 0 = filthy, 100 = spotless
}

/// A stat value that enforces bounds (0-100)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StatValue(u8);

impl StatValue {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 100;
    
    pub fn new(value: u8) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }
    
    pub fn add(&mut self, amount: i16) {
        let new_value = (self.0 as i16 + amount).clamp(0, 100) as u8;
        self.0 = new_value;
    }
    
    pub fn value(&self) -> u8 { self.0 }
    pub fn percentage(&self) -> f32 { self.0 as f32 / 100.0 }
}
```

### Life Stage State Machine

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeStage {
    Egg,
    Baby,
    Child,
    Teen,
    Adult,
}

impl LifeStage {
    /// Age thresholds in seconds for stage transitions
    pub const THRESHOLDS: [(LifeStage, u64); 4] = [
        (LifeStage::Baby, 60),      // 1 minute
        (LifeStage::Child, 300),    // 5 minutes
        (LifeStage::Teen, 600),     // 10 minutes
        (LifeStage::Adult, 1200),   // 20 minutes
    ];
    
    pub fn next(&self) -> Option<LifeStage> {
        match self {
            LifeStage::Egg => Some(LifeStage::Baby),
            LifeStage::Baby => Some(LifeStage::Child),
            LifeStage::Child => Some(LifeStage::Teen),
            LifeStage::Teen => Some(LifeStage::Adult),
            LifeStage::Adult => None,
        }
    }
}
```

### Pet Behavior State Machine

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetState {
    Normal,
    Sleeping { since: DateTime<Utc> },
    Sick { since: DateTime<Utc> },
    Dead { at: DateTime<Utc> },
}

impl PetState {
    /// Check if pet can perform an action
    pub fn can(&self, action: Action) -> bool {
        match self {
            PetState::Normal => true,
            PetState::Sleeping { .. } => matches!(action, Action::Wake),
            PetState::Sick { .. } => !matches!(action, Action::Play),
            PetState::Dead { .. } => false,
        }
    }
    
    /// Check if state transition is valid
    pub fn can_transition_to(&self, new_state: PetState) -> bool {
        match (self, new_state) {
            (PetState::Dead { .. }, _) => false,  // Dead is terminal
            (PetState::Sleeping { .. }, PetState::Normal) => true,
            (PetState::Sick { .. }, PetState::Normal) => true,
            (PetState::Normal, _) => true,
            _ => false,
        }
    }
}
```

### Game State Management

The `GameState` enum manages the overall game flow, including the egg stage, normal gameplay, and game over:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    /// Initial state - showing title screen/menu
    Menu,
    
    /// Egg incubation phase
    /// Player must warm the egg and wait for it to hatch
    Egg,
    
    /// Normal gameplay with a hatched pet
    /// Pet can be fed, played with, cleaned, etc.
    Playing,
    
    /// Game is paused
    Paused,
    
    /// Game over - pet died
    GameOver { reason: DeathReason },
}

/// Reasons for game over
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathReason {
    /// Egg died from neglect (cold exposure)
    EggDied,
    /// Pet died from health reaching 0
    HealthDepleted,
    /// Pet died from starvation
    Starvation,
}

impl GameState {
    /// Check if game is in an active state (not paused/menu/game over)
    pub fn is_active(&self) -> bool {
        matches!(self, GameState::Egg | GameState::Playing)
    }
    
    /// Check if input should be processed
    pub fn accepts_input(&self) -> bool {
        matches!(self, GameState::Egg | GameState::Playing | GameState::Menu)
    }
    
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            GameState::Menu => "Menu",
            GameState::Egg => "Incubating Egg",
            GameState::Playing => "Playing",
            GameState::Paused => "Paused",
            GameState::GameOver { .. } => "Game Over",
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Menu
    }
}
```

### Game State Transitions

State transitions follow specific rules based on game events:

```rust
impl App {
    /// Handle state transitions based on game events
    pub fn update_game_state(&mut self) {
        match self.game_state {
            GameState::Egg => {
                // Check for game over (egg died)
                if self.pet.egg_stats.is_dead() {
                    self.transition_to(GameState::GameOver { 
                        reason: DeathReason::EggDied 
                    });
                    return;
                }
                
                // Check for hatching
                if self.pet.egg_stats.is_ready_to_hatch() {
                    self.pet.hatch();
                    self.transition_to(GameState::Playing);
                    
                    // Show hatching notification
                    self.event_log.add("The egg has hatched! Welcome, baby!");
                }
            }
            
            GameState::Playing => {
                // Check for game over (pet died)
                if !self.pet.is_alive() {
                    let reason = if self.pet.stats.hunger.value() >= 100 {
                        DeathReason::Starvation
                    } else {
                        DeathReason::HealthDepleted
                    };
                    self.transition_to(GameState::GameOver { reason });
                }
            }
            
            // Terminal states - no transitions
            GameState::GameOver { .. } => {}
            GameState::Menu => {}
            GameState::Paused => {}
        }
    }
    
    /// Perform a state transition with validation
    pub fn transition_to(&mut self, new_state: GameState) {
        // Validate transition
        if !self.can_transition_to(&new_state) {
            log::warn!("Invalid state transition: {:?} -> {:?}", 
                      self.game_state, new_state);
            return;
        }
        
        log::info!("Game state: {:?} -> {:?}", 
                  self.game_state, new_state);
        
        let old_state = self.game_state;
        self.game_state = new_state;
        
        // Handle transition side effects
        match (old_state, new_state) {
            (_, GameState::Playing) => {
                // Start gameplay music/ambience
                self.start_gameplay();
            }
            (GameState::Playing, GameState::Paused) => {
                // Pause timers
                self.pause_game_loop();
            }
            (_, GameState::GameOver { reason }) => {
                // Save final stats, show game over screen
                self.handle_game_over(reason);
            }
            _ => {}
        }
    }
    
    /// Check if a state transition is valid
    fn can_transition_to(&self, new_state: &GameState) -> bool {
        use GameState::*;
        
        match (&self.game_state, new_state) {
            // Menu can go to Egg (new game) or Exit
            (Menu, Egg) => true,
            (Menu, Playing) => false, // Must go through Egg first
            
            // Egg can hatch into Playing or die to GameOver
            (Egg, Playing) => true,
            (Egg, GameOver { .. }) => true,
            
            // Playing can pause or end
            (Playing, Paused) => true,
            (Playing, GameOver { .. }) => true,
            
            // Paused can resume
            (Paused, Playing) => true,
            
            // GameOver can go to Menu (restart)
            (GameOver { .. }, Menu) => true,
            
            // Same state is always allowed (idempotent)
            (a, b) if std::mem::discriminant(a) == std::mem::discriminant(b) => true,
            
            _ => false,
        }
    }
}
```

**Valid State Transitions:**

```
┌─────────┐     ┌─────┐     ┌─────────┐
│  Menu   │────▶│ Egg │────▶│ Playing │
└────┬────┘     └─────┘     └────┬────┘
     │          │    │          │
     │          │    └──────────┤
     │          ▼               ▼
     │      ┌─────────┐    ┌─────────┐
     └─────▶│ GameOver│◄───┘ Paused  │
            └─────────┘    └─────────┘
```

- **Menu** → Egg: Start new game
- **Egg** → Playing: Egg hatched successfully
- **Egg** → GameOver: Egg died from neglect
- **Playing** → Paused: Player pauses game
- **Playing** → GameOver: Pet died
- **Paused** → Playing: Resume game
- **GameOver** → Menu: Return to menu (can start new game)

### Game Over Handling

When the game ends, the system handles cleanup and finalization:

```rust
impl App {
    /// Handle game over state
    fn handle_game_over(&mut self, reason: DeathReason) {
        // Final save (for statistics/history)
        if let Err(e) = self.save_final_stats() {
            log::error!("Failed to save final stats: {}", e);
        }
        
        // Generate game over message
        let message = match reason {
            DeathReason::EggDied => {
                "The egg grew cold and lifeless...".to_string()
            }
            DeathReason::HealthDepleted => {
                format!("{} passed away from poor health...", self.pet.name)
            }
            DeathReason::Starvation => {
                format!("{} starved to death...", self.pet.name)
            }
        };
        
        // Add to event log
        self.event_log.add(&message);
        
        // Stop game loop
        self.stop_game_loop();
        
        // Show game over UI
        self.show_game_over_screen(&message, reason);
    }
    
    /// Show game over screen with stats and options
    fn show_game_over_screen(&mut self, message: &str, reason: DeathReason) {
        self.ui_state.show_popup = Some(Popup::GameOver {
            message: message.to_string(),
            reason,
            play_time: self.total_play_time,
            pet_age: self.pet.age(),
            final_stats: self.pet.stats,
        });
    }
    
    /// Restart the game (from game over)
    pub fn restart_game(&mut self) {
        // Clean up old pet
        self.cleanup_save_file();
        
        // Reset to initial state
        self.pet = Pet::new("Fluffy");
        self.game_state = GameState::Egg;
        self.total_play_time = Duration::ZERO;
        self.event_log.clear();
        
        // Start fresh
        self.start_game_loop();
    }
}
```

**Game Over Screen Display:**

```
┌─────────────────────────────────────────────┐
│                GAME OVER                    │
├─────────────────────────────────────────────┤
│                                             │
│     The egg grew cold and lifeless...       │
│                                             │
│     Play Time: 2m 15s                       │
│     Egg Health: 0%                          │
│                                             │
│     [R] Restart    [M] Menu    [Q] Quit     │
│                                             │
└─────────────────────────────────────────────┘
```

## State Update Patterns

### 1. Command Pattern for Actions

```rust
pub trait Command {
    fn execute(&self, pet: &mut Pet) -> Result<Vec<GameEvent>, CommandError>;
    fn undo(&self, pet: &mut Pet) -> Result<(), CommandError>;
}

pub struct FeedCommand { amount: u8 }

impl Command for FeedCommand {
    fn execute(&self, pet: &mut Pet) -> Result<Vec<GameEvent>, CommandError> {
        if !pet.state.can(Action::Feed) {
            return Err(CommandError::InvalidState);
        }
        
        let old_hunger = pet.stats.hunger.value();
        pet.stats.hunger.add(-(self.amount as i16));
        pet.stats.energy.add(-5);  // Digestion costs energy
        
        let events = vec![
            GameEvent::StatChanged {
                stat: StatType::Hunger,
                old: old_hunger,
                new: pet.stats.hunger.value(),
            }
        ];
        
        Ok(events)
    }
    
    fn undo(&self, pet: &mut Pet) -> Result<(), CommandError> {
        pet.stats.hunger.add(self.amount as i16);
        Ok(())
    }
}
```

### 2. Time-Based State Decay

```rust
impl Pet {
    /// Update stats based on elapsed time
    pub fn update_time_based(&mut self, elapsed: Duration) {
        let seconds = elapsed.as_secs_f32();
        
        // Decay rates (per second)
        const HUNGER_DECAY: f32 = 0.5;    // +0.5 hunger per second
        const HAPPINESS_DECAY: f32 = 0.2; // -0.2 happiness per second
        const ENERGY_RECOVERY: f32 = 0.3; // +0.3 energy per second (if sleeping)
        const HYGIENE_DECAY: f32 = 0.1;   // +0.1 dirtiness per second
        
        match self.state {
            PetState::Normal => {
                self.stats.hunger.add((seconds * HUNGER_DECAY) as i16);
                self.stats.happiness.add(-(seconds * HAPPINESS_DECAY) as i16);
                self.stats.hygiene.add((seconds * HYGIENE_DECAY) as i16);
            }
            PetState::Sleeping { .. } => {
                self.stats.energy.add((seconds * ENERGY_RECOVERY) as i16);
                self.stats.hunger.add((seconds * HUNGER_DECAY * 0.5) as i16); // Slower when sleeping
            }
            PetState::Sick { .. } => {
                self.stats.health.add(-(seconds * 0.1) as i16);
            }
            PetState::Dead { .. } => {}
        }
        
        // Check for state transitions based on stats
        self.check_state_transitions();
    }
    
    fn check_state_transitions(&mut self) {
        // Wake up if fully rested
        if let PetState::Sleeping { since } = self.state {
            if self.stats.energy.value() >= 95 {
                self.state = PetState::Normal;
                // Trigger wake up animation
            }
        }
        
        // Get sick if neglected
        if self.stats.hygiene.value() < 20 || self.stats.hunger.value() > 90 {
            if matches!(self.state, PetState::Normal) && fastrand::f32() < 0.001 {
                self.state = PetState::Sick { since: Utc::now() };
            }
        }
        
        // Die if health reaches zero
        if self.stats.health.value() == 0 {
            self.state = PetState::Dead { at: Utc::now() };
        }
    }
}
```

### 3. Immutable Updates with Lens Pattern

For predictable state updates, use lens-style accessors:

```rust
impl Stats {
    /// Returns updated stats without mutating original
    pub fn with_hunger(self, value: u8) -> Self {
        let mut new = self;
        new.hunger = StatValue::new(value);
        new
    }
    
    /// Functional update: map a stat through a function
    pub fn map_hunger<F>(self, f: F) -> Self 
    where F: FnOnce(u8) -> u8 
    {
        self.with_hunger(f(self.hunger.value()))
    }
}

// Usage
let new_stats = pet.stats
    .map_hunger(|h| h.saturating_sub(20))
    .map_happiness(|h| h.saturating_add(10));
```

## State Persistence

### Save Data Format

```rust
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub saved_at: DateTime<Utc>,
    pub pet: Pet,
    pub play_time: Duration,
    pub achievements: Vec<Achievement>,
}

impl SaveData {
    pub const CURRENT_VERSION: u32 = 1;
    
    pub fn migrate(&mut self) {
        match self.version {
            1 => { /* Current version, no migration needed */ }
            _ => {
                log::warn!("Unknown save version {}, attempting best-effort load", self.version);
            }
        }
    }
}
```

### Auto-Save Strategy

```rust
pub struct AutoSave {
    last_save: Instant,
    dirty: bool,
    interval: Duration,
}

impl AutoSave {
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    pub fn should_save(&self) -> bool {
        self.dirty && self.last_save.elapsed() >= self.interval
    }
    
    pub fn saved(&mut self) {
        self.last_save = Instant::now();
        self.dirty = false;
    }
}
```

## State Validation

### Invariants

```rust
impl Pet {
    /// Verify all state invariants (call after loading saves)
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Stat bounds
        if self.stats.hunger.value() > 100 {
            return Err(ValidationError::StatOutOfBounds("hunger"));
        }
        
        // State consistency
        if matches!(self.state, PetState::Dead { .. }) && self.stats.health.value() > 0 {
            return Err(ValidationError::InconsistentState);
        }
        
        // Age vs stage consistency
        let expected_stage = LifeStage::from_age(self.age());
        if self.stage != expected_stage && !self.is_transitioning() {
            return Err(ValidationError::StageMismatch);
        }
        
        Ok(())
    }
}
```

## UI State Separation

Keep UI state separate from game state:

```rust
pub struct App {
    // Game state (saved)
    pub pet: Pet,
    
    // UI state (not saved)
    pub ui: UiState,
}

pub struct UiState {
    pub selected_action: usize,
    pub show_help: bool,
    pub event_log_scroll: u16,
    pub current_popup: Option<Popup>,
}
```

## Thread Safety

```rust
// Main thread owns the state
pub struct App {
    pet: Pet,
}

// Share across threads with Arc<Mutex<_>>
let app = Arc::new(Mutex::new(App::new()));

// Game loop thread
let app_clone = Arc::clone(&app);
tokio::spawn(async move {
    loop {
        sleep(Duration::from_secs(1)).await;
        let mut app = app_clone.lock().await;
        app.tick();
    }
});
```
