# Phase 4: Game Loop

**Status**: 📋 Not Started  
**Duration**: Week 2-3  
**Goal**: Real-time stat decay, aging, events, and death mechanics

## Overview

This phase implements the continuous game loop: stats decay over time, the pet ages, random events occur, and state transitions happen automatically. The pet can die from neglect.

## Prerequisites

- Phase 3 complete (actions working)
- Understanding of tokio async
- Understanding of timers and intervals

## Step 1: Tokio Async Setup

Update `Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
tokio = { version = "1.0", features = ["full"] }
tokio-util = "0.7"
```

Update `src/main.rs` for async:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let mut app = App::new();
    let mut terminal = tui::init()?;
    
    // Run async event loop
    let result = run_app_async(&mut app, &mut terminal).await;
    
    tui::restore(&mut terminal)?;
    
    result
}
```

## Step 2: Game Loop Implementation

Create `src/game_loop.rs`:

```rust
use std::sync::Arc;
use std::time::Duration;

use tokio::{
    sync::Mutex,
    time::{interval, Interval},
};

use crate::app::App;
use crate::pet::{LifeStage, PetState};
use crate::event_log::LogCategory;
use chrono::Utc;

/// Game loop configuration
#[derive(Debug, Clone)]
pub struct GameLoopConfig {
    /// How often to update stats (seconds)
    pub tick_rate_secs: u64,
    /// How often to check for random events (seconds)
    pub event_check_rate_secs: u64,
    /// How often to auto-save (seconds)
    pub auto_save_rate_secs: u64,
}

impl Default for GameLoopConfig {
    fn default() -> Self {
        Self {
            tick_rate_secs: 1,         // Every second
            event_check_rate_secs: 30, // Every 30 seconds
            auto_save_rate_secs: 60,   // Every minute
        }
    }
}

/// Decay rates (points per second)
#[derive(Debug, Clone)]
pub struct DecayRates {
    pub hunger: f32,      // Increases over time (gets hungry)
    pub happiness: f32,   // Decreases (gets sad)
    pub energy: f32,      // Decreases when awake, increases when sleeping
    pub hygiene: f32,     // Increases (gets dirty)
    pub health: f32,      // Decreases if neglected
}

impl Default for DecayRates {
    fn default() -> Self {
        Self {
            hunger: 0.5,      // +0.5 per second = 50 per 100 seconds
            happiness: 0.2,   // -0.2 per second
            energy: 0.15,     // -0.15 when awake
            hygiene: 0.1,     // +0.1 per second
            health: 0.0,      // Only decreases when neglected
        }
    }
}

/// Game loop runner
pub struct GameLoop {
    app: Arc<Mutex<App>>,
    config: GameLoopConfig,
    decay_rates: DecayRates,
}

impl GameLoop {
    pub fn new(app: App, config: GameLoopConfig) -> Self {
        Self {
            app: Arc::new(Mutex::new(app)),
            config,
            decay_rates: DecayRates::default(),
        }
    }
    
    pub fn app_handle(&self) -> Arc<Mutex<App>> {
        Arc::clone(&self.app)
    }
    
    pub async fn run(&self) {
        let mut tick_interval = interval(Duration::from_secs(self.config.tick_rate_secs));
        let mut event_interval = interval(Duration::from_secs(self.config.event_check_rate_secs));
        let mut save_interval = interval(Duration::from_secs(self.config.auto_save_rate_secs));
        
        // Random seed
        fastrand::seed(Utc::now().timestamp() as u64);
        
        loop {
            tokio::select! {
                // Game tick - update stats and aging
                _ = tick_interval.tick() => {
                    let mut app = self.app.lock().await;
                    
                    if !app.running {
                        break;
                    }
                    
                    self.tick(&mut app).await;
                }
                
                // Random event check
                _ = event_interval.tick() => {
                    let mut app = self.app.lock().await;
                    self.check_random_events(&mut app).await;
                }
                
                // Auto-save
                _ = save_interval.tick() => {
                    // Save logic here
                    let mut app = self.app.lock().await;
                    app.event_log.info("Game auto-saved");
                }
            }
        }
    }
    
    async fn tick(&self, app: &mut App) {
        use std::time::Duration;
        
        let dt = Duration::from_secs(1);
        let dt_secs = dt.as_secs_f32();
        
        // Update age
        let old_stage = app.pet.stage;
        if let Some(new_stage) = app.pet.update_age(dt) {
            app.event_log.success(format!(
                "{} evolved into a {:?}!",
                app.pet.name, new_stage
            ));
        }
        
        // Apply decay based on state
        match app.pet.state {
            PetState::Normal => {
                // Hunger increases
                let hunger_delta = (self.decay_rates.hunger * dt_secs) as i16;
                app.pet.stats.hunger.add(hunger_delta);
                
                // Happiness decreases slowly
                let happy_delta = -(self.decay_rates.happiness * dt_secs) as i16;
                app.pet.stats.happiness.add(happy_delta);
                
                // Energy decreases
                let energy_delta = -(self.decay_rates.energy * dt_secs) as i16;
                app.pet.stats.energy.add(energy_delta);
                
                // Hygiene decreases
                let hygiene_delta = (self.decay_rates.hygiene * dt_secs) as i16;
                app.pet.stats.hygiene.add(hygiene_delta);
            }
            
            PetState::Sleeping { since } => {
                // Energy regenerates faster while sleeping
                app.pet.stats.energy.add((0.5 * dt_secs) as i16);
                
                // Hunger increases slower
                app.pet.stats.hunger.add((self.decay_rates.hunger * 0.3 * dt_secs) as i16);
                
                // Auto-wake if fully rested
                if app.pet.stats.energy.value() >= 95 {
                    app.pet.state = PetState::Normal;
                    app.event_log.info(format!("{} woke up naturally (fully rested)", app.pet.name));
                }
            }
            
            PetState::Sick { .. } => {
                // Health decreases while sick
                app.pet.stats.health.add(-(0.1 * dt_secs) as i16);
                
                // All other stats decay faster
                app.pet.stats.hunger.add((self.decay_rates.hunger * 1.5 * dt_secs) as i16);
                app.pet.stats.happiness.add(-(self.decay_rates.happiness * 2.0 * dt_secs) as i16);
            }
            
            PetState::Dead { .. } => {
                // No decay when dead
                app.running = false;
            }
        }
        
        // Check for sickness (chance increases with bad stats)
        self.check_sickness(app);
        
        // Check for death
        self.check_death(app);
        
        // Log warnings for critical stats
        self.check_warnings(app);
        
        // Increment tick count
        app.tick_count += 1;
    }
    
    fn check_sickness(&self, app: &mut App) {
        // Can't get sick if already sick or dead
        if !matches!(app.pet.state, PetState::Normal) {
            return;
        }
        
        // Calculate sickness chance based on stats
        let mut chance = 0.0f32;
        
        if app.pet.stats.hygiene.value() < 20 {
            chance += 0.001; // 0.1% per tick when dirty
        }
        if app.pet.stats.hunger.value() > 80 {
            chance += 0.002; // 0.2% per tick when starving
        }
        if app.pet.stats.happiness.value() < 10 {
            chance += 0.001; // Stress lowers immunity
        }
        
        // Roll for sickness
        if fastrand::f32() < chance {
            app.pet.state = PetState::Sick { since: Utc::now() };
            app.event_log.warning(format!(
                "{} got sick! Give them medicine!",
                app.pet.name
            ));
        }
    }
    
    fn check_death(&self, app: &mut App) {
        // Health reaches 0
        if app.pet.stats.health.value() == 0 {
            app.pet.state = PetState::Dead { at: Utc::now() };
            app.event_log.error(format!(
                "{} has passed away...",
                app.pet.name
            ));
            return;
        }
        
        // Multiple critical stats
        let critical_count = [
            app.pet.stats.hunger.value() == 100,
            app.pet.stats.happiness.value() == 0,
            app.pet.stats.energy.value() == 0,
        ].iter().filter(|&&x| x).count();
        
        if critical_count >= 2 {
            // Chance of death increases with neglect
            let death_chance = 0.001 * critical_count as f32;
            if fastrand::f32() < death_chance {
                app.pet.stats.health = 0.into();
                app.pet.state = PetState::Dead { at: Utc::now() };
                app.event_log.error(format!(
                    "{} died from neglect...",
                    app.pet.name
                ));
            }
        }
    }
    
    fn check_warnings(&self, app: &mut App) {
        // Warn about critical stats (once per threshold crossing)
        // Implementation would track previous state to avoid spam
        
        if app.pet.stats.hunger.value() > 80 && app.pet.stats.hunger.value() <= 85 {
            app.event_log.warning(format!("{} is getting very hungry!", app.pet.name));
        }
        
        if app.pet.stats.energy.value() < 20 && app.pet.stats.energy.value() >= 15 {
            app.event_log.warning(format!("{} is getting very tired!", app.pet.name));
        }
    }
    
    async fn check_random_events(&self, app: &mut App) {
        // Only trigger events when pet is alive and normal
        if !matches!(app.pet.state, PetState::Normal) {
            return;
        }
        
        // 10% chance of a random event per check
        if fastrand::f32() >= 0.1 {
            return;
        }
        
        // Possible events
        let events = [
            ("found_a_treat", "{} found a treat on the floor! (+10 happiness)"),
            ("made_a_mess", "{} made a mess... (-20 hygiene)"),
            ("had_a_dream", "{} had a good dream! (+5 happiness)"),
            ("sunny_day", "It's a beautiful day! (+5 happiness)"),
        ];
        
        let (event_id, message) = events[fastrand::usize(..events.len())];
        
        // Apply event effects
        match event_id {
            "found_a_treat" => {
                app.pet.stats.happiness.add(10);
                app.pet.stats.hunger.add(-5);
            }
            "made_a_mess" => {
                app.pet.stats.hygiene.add(-20);
            }
            "had_a_dream" | "sunny_day" => {
                app.pet.stats.happiness.add(5);
            }
            _ => {}
        }
        
        app.event_log.info(message.replace("{}", &app.pet.name));
    }
}
```

## Step 3: Update Event Handler for Async

Update `src/event.rs`:

```rust
use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyEvent};
use tokio::time::{interval, Interval};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Tick,
    Key(KeyEvent),
}

/// Async event handler
pub struct EventHandler {
    tick_interval: Interval,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            tick_interval: interval(tick_rate),
        }
    }
    
    /// Get next event (async)
    pub async fn next(&mut self) -> Result<Event> {
        tokio::select! {
            _ = self.tick_interval.tick() => {
                Ok(Event::Tick)
            }
            
            result = tokio::task::spawn_blocking(|| event::read()) => {
                match result {
                    Ok(Ok(CEvent::Key(key))) => Ok(Event::Key(key)),
                    Ok(Ok(_)) => Ok(Event::Tick), // Ignore other events
                    Ok(Err(e)) => Err(AppError::Terminal(e.to_string())),
                    Err(e) => Err(AppError::Terminal(e.to_string())),
                }
            }
        }
    }
}
```

## Step 4: Update App for Async

Update `src/app.rs`:

```rust
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub tick_count: u64,
    pub pet: Pet,
    pub art_cache: AsciiArtCache,
    pub event_log: EventLog,
    pub current_animation: Option<AnimationTrigger>,
    pub last_update: std::time::Instant,
}

impl App {
    pub fn time_since_last_update(&self) -> std::time::Duration {
        self.last_update.elapsed()
    }
    
    pub fn mark_updated(&mut self) {
        self.last_update = std::time::Instant::now();
    }
}
```

## Step 5: Main Async Loop

Update `src/main.rs`:

```rust
use std::time::Duration;

use game_loop::{GameLoop, GameLoopConfig};

async fn run_app_async(app: &mut App, terminal: &mut tui::Tui) -> Result<()> {
    use tokio::sync::Mutex;
    
    // Wrap app in Arc<Mutex> for sharing
    let app = Arc::new(Mutex::new(std::mem::take(app)));
    
    // Create game loop
    let game_loop = GameLoop::new(
        // We need to handle this differently - maybe App should be Clone?
        // For now, we'll use a simpler approach
        todo!("Initialize game loop properly"),
        GameLoopConfig::default(),
    );
    
    // Spawn game loop task
    let game_handle = tokio::spawn(async move {
        game_loop.run().await;
    });
    
    // UI loop in main thread
    let mut event_handler = event::EventHandler::new(Duration::from_millis(100));
    
    while app.lock().await.running {
        // Draw
        let app_guard = app.lock().await;
        terminal.draw(|f| ui::draw(f, &app_guard))?;
        drop(app_guard); // Release lock
        
        // Handle input
        if let Ok(Event::Key(key)) = event_handler.next().await {
            let mut app_guard = app.lock().await;
            app_guard.handle_key(key);
        }
    }
    
    // Wait for game loop to finish
    let _ = game_handle.await;
    
    Ok(())
}
```

**Note**: The async architecture needs careful handling of App sharing. Consider:
1. Making App Clone-able (derive Clone)
2. Or using message passing between game loop and UI
3. Or using channels for all state updates

## Simplified Architecture (Alternative)

Instead of Arc<Mutex<App>>, use message passing:

```rust
// Game loop sends updates via channel
enum GameUpdate {
    StatDecay,
    AgeAdvance,
    RandomEvent(String),
    PetDied,
}

// Main loop receives updates and applies them
```

## Phase 4 Success Criteria

- [x] Stats decay automatically over time
- [x] Pet ages automatically
- [x] Life stage transitions happen at thresholds
- [x] Sleeping regenerates energy
- [x] Pet can get sick from neglect
- [x] Pet can die from critical neglect
- [x] Random events occur periodically
- [x] Auto-save triggers (placeholder for Phase 6)
- [x] Game runs at ~1 FPS for updates, faster for input

## Configuration

Allow adjusting time scale for testing:

```rust
pub struct TimeScale {
    pub multiplier: f32,  // 1.0 = real time, 60.0 = 1 sec = 1 minute
}

impl Default for TimeScale {
    fn default() -> Self {
        Self { multiplier: 60.0 }  // Accelerated by default for gameplay
    }
}
```

## Next Steps

Move to **Phase 5: Animation** for visual feedback and life.
