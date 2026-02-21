# Phase 2: Pet System

**Status**: 📋 Not Started  
**Duration**: Week 1-2  
**Goal**: Core pet data structures with stats, life stages, and basic display

## Overview

This phase implements the core pet system: the Pet struct, Stats system with value clamping, life stage progression, and time-based aging. By the end, you'll have a pet with changing stats that can be displayed in the terminal.

## Prerequisites

- Phase 1 complete (working Ratatui app)
- Understanding of Rust enums and structs
- Basic understanding of time calculations

## Step 1: Stat Value System

Create `src/stats.rs`:

```rust
use serde::{Deserialize, Serialize};

/// A stat value bounded between 0 and 100
/// 
/// 0 is the "worst" state (starving for hunger, dead for health)
/// 100 is the "best" state (full for hunger, perfect for health)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StatValue(u8);

impl StatValue {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 100;
    
    /// Create a new stat value, clamped to valid range
    pub fn new(value: u8) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }
    
    /// Get the current value
    pub fn value(&self) -> u8 {
        self.0
    }
    
    /// Get as percentage (0.0 - 1.0)
    pub fn percentage(&self) -> f32 {
        self.0 as f32 / 100.0
    }
    
    /// Add a value (can be negative), clamped to bounds
    pub fn add(&mut self, delta: i16) {
        let new_value = (self.0 as i16 + delta).clamp(0, 100) as u8;
        self.0 = new_value;
    }
    
    /// Check if stat is in critical state (< 25)
    pub fn is_critical(&self) -> bool {
        self.0 < 25
    }
    
    /// Check if stat is good (> 75)
    pub fn is_good(&self) -> bool {
        self.0 > 75
    }
}

impl From<u8> for StatValue {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

impl PartialEq<u8> for StatValue {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

/// All pet stats
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Stats {
    /// Hunger: 0 = full, 100 = starving
    pub hunger: StatValue,
    /// Happiness: 0 = depressed, 100 = ecstatic
    pub happiness: StatValue,
    /// Energy: 0 = exhausted, 100 = energetic
    pub energy: StatValue,
    /// Health: 0 = dead, 100 = perfect
    pub health: StatValue,
    /// Hygiene: 0 = filthy, 100 = spotless
    pub hygiene: StatValue,
}

impl Stats {
    /// Create new stats with default values
    pub fn new() -> Self {
        Self {
            hunger: StatValue::new(50),      // Slightly hungry
            happiness: StatValue::new(70),   // Happy
            energy: StatValue::new(80),      // Well rested
            health: StatValue::new(100),     // Perfect health
            hygiene: StatValue::new(80),     // Clean
        }
    }
    
    /// Create stats for a new egg
    pub fn egg() -> Self {
        Self {
            hunger: StatValue::new(0),
            happiness: StatValue::new(50),
            energy: StatValue::new(50),
            health: StatValue::new(100),
            hygiene: StatValue::new(100),
        }
    }
    
    /// Get the worst stat (for alerts)
    pub fn worst_stat(&self) -> (&'static str, u8) {
        let stats = [
            ("Hunger", self.hunger.value()),
            ("Happiness", self.happiness.value()),
            ("Energy", self.energy.value()),
            ("Health", self.health.value()),
            ("Hygiene", self.hygiene.value()),
        ];
        
        stats.iter()
            .min_by_key(|(_, v)| v)
            .map(|(n, v)| (*n, *v))
            .unwrap_or(("Unknown", 0))
    }
    
    /// Check if any stat is critical
    pub fn has_critical(&self) -> bool {
        self.hunger.is_critical() ||
        self.happiness.is_critical() ||
        self.energy.is_critical() ||
        self.health.is_critical() ||
        self.hygiene.is_critical()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stat_bounds() {
        let mut stat = StatValue::new(50);
        
        stat.add(100);
        assert_eq!(stat.value(), 100);
        
        stat.add(-200);
        assert_eq!(stat.value(), 0);
    }
    
    #[test]
    fn test_stat_clamping_on_create() {
        let stat = StatValue::new(150);
        assert_eq!(stat.value(), 100);
        
        let stat = StatValue::new(0);
        assert_eq!(stat.value(), 0);
    }
}
```

## Step 2: Life Stage Enum

Add to `src/pet.rs` (create file):

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Pet life stages in order of progression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeStage {
    Egg,
    Baby,
    Child,
    Teen,
    Adult,
}

impl LifeStage {
    /// Time thresholds for each stage (in seconds)
    pub fn threshold(&self) -> Duration {
        match self {
            LifeStage::Egg => Duration::from_secs(0),
            LifeStage::Baby => Duration::from_secs(60),     // 1 minute
            LifeStage::Child => Duration::from_secs(300),   // 5 minutes
            LifeStage::Teen => Duration::from_secs(600),    // 10 minutes
            LifeStage::Adult => Duration::from_secs(1200),  // 20 minutes
        }
    }
    
    /// Get the next stage
    pub fn next(&self) -> Option<LifeStage> {
        match self {
            LifeStage::Egg => Some(LifeStage::Baby),
            LifeStage::Baby => Some(LifeStage::Child),
            LifeStage::Child => Some(LifeStage::Teen),
            LifeStage::Teen => Some(LifeStage::Adult),
            LifeStage::Adult => None,
        }
    }
    
    /// Get ASCII art filename for this stage
    pub fn art_file(&self) -> &'static str {
        match self {
            LifeStage::Egg => "egg.txt",
            LifeStage::Baby => "baby.txt",
            LifeStage::Child => "child.txt",
            LifeStage::Teen => "teen.txt",
            LifeStage::Adult => "adult.txt",
        }
    }
    
    /// Display name
    pub fn name(&self) -> &'static str {
        match self {
            LifeStage::Egg => "Egg",
            LifeStage::Baby => "Baby",
            LifeStage::Child => "Child",
            LifeStage::Teen => "Teen",
            LifeStage::Adult => "Adult",
        }
    }
}

impl Default for LifeStage {
    fn default() -> Self {
        LifeStage::Egg
    }
}
```

## Step 2.5: Stage-Specific Mechanics

### Egg Stage Mechanics

The Egg stage is a unique interactive phase where players must incubate the egg before it hatches. Unlike later stages, eggs don't have traditional stats - instead, they use a specialized `EggStats` system.

#### EggStats Struct

```rust
use serde::{Deserialize, Serialize};

/// Statistics for the egg incubation phase
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EggStats {
    /// Incubation progress (0-100%)
    /// Egg hatches when this reaches 100%
    pub incubation_progress: f32,
    
    /// Warmth level (0-100)
    /// Maintaining warmth is critical for egg health
    pub warmth_level: u8,
    
    /// Egg health (0-100)
    /// Only visible when warmth drops below critical threshold
    /// If health reaches 0, the game is over
    pub health: u8,
}

impl Default for EggStats {
    fn default() -> Self {
        Self {
            incubation_progress: 0.0,
            warmth_level: 50,  // Start at moderate warmth
            health: 100,       // Full health initially
        }
    }
}

impl EggStats {
    /// Check if the egg has finished incubating
    pub fn is_ready_to_hatch(&self) -> bool {
        self.incubation_progress >= 100.0
    }
    
    /// Check if the egg is dead (game over)
    pub fn is_dead(&self) -> bool {
        self.health == 0
    }
    
    /// Check if health should be visible (warmth below critical threshold)
    pub fn should_show_health(&self) -> bool {
        self.warmth_level < 30
    }
}
```

#### Incubation Mechanics

The egg incubates over time automatically:

```rust
impl Pet {
    /// Update egg incubation progress
    /// Called every tick when in Egg stage
    pub fn update_incubation(&mut self, delta: Duration) {
        if self.stage != LifeStage::Egg {
            return;
        }
        
        // Base incubation rate: 100% over 30 seconds
        const INCUBATION_TIME: f32 = 30.0; // seconds
        const PROGRESS_PER_SECOND: f32 = 100.0 / INCUBATION_TIME;
        
        let progress = delta.as_secs_f32() * PROGRESS_PER_SECOND;
        self.egg_stats.incubation_progress = 
            (self.egg_stats.incubation_progress + progress).min(100.0);
    }
    
    /// Check if egg should hatch
    pub fn check_hatching(&mut self) -> bool {
        if self.stage != LifeStage::Egg {
            return false;
        }
        
        if self.egg_stats.is_ready_to_hatch() {
            self.hatch();
            return true;
        }
        false
    }
}
```

**Incubation Rules:**
- Total incubation time: **30 seconds** (configurable)
- Progress increases automatically over time
- Warmth level affects egg health but not incubation speed
- Egg hatches automatically when progress reaches 100%

#### Warmth System

Players must actively maintain the egg's warmth:

```rust
impl Pet {
    /// Warm the egg by pressing 'W'
    /// Each press adds +10 warmth (clamped to 100 max)
    pub fn warm_egg(&mut self) -> Result<WarmingResult, EggError> {
        if self.stage != LifeStage::Egg {
            return Err(EggError::NotAnEgg);
        }
        
        if self.egg_stats.is_dead() {
            return Err(EggError::EggDead);
        }
        
        // Add warmth (clamped to 100)
        let old_warmth = self.egg_stats.warmth_level;
        self.egg_stats.warmth_level = 
            (self.egg_stats.warmth_level + 10).min(100);
        
        // If warmth was very low, recover some health
        if old_warmth < 30 {
            self.egg_stats.health = (self.egg_stats.health + 5).min(100);
        }
        
        Ok(WarmingResult {
            previous_warmth: old_warmth,
            current_warmth: self.egg_stats.warmth_level,
        })
    }
    
    /// Decay warmth over time
    /// Called periodically (every 5 seconds)
    pub fn decay_warmth(&mut self) {
        if self.stage != LifeStage::Egg {
            return;
        }
        
        // Warmth decays by 3 points every 5 seconds
        const WARMTH_DECAY: u8 = 3;
        const DECAY_INTERVAL: Duration = Duration::from_secs(5);
        
        self.egg_stats.warmth_level = 
            self.egg_stats.warmth_level.saturating_sub(WARMTH_DECAY);
    }
}

/// Result of a warming action
#[derive(Debug, Clone, Copy)]
pub struct WarmingResult {
    pub previous_warmth: u8,
    pub current_warmth: u8,
}
```

**Warmth Mechanics:**
- **Press W**: Instantly adds +10 warmth (clamped to max 100)
- **Decay rate**: -3 warmth every 5 seconds
- **Equilibrium**: To maintain warmth, press W roughly every 15 seconds
- **Range**: 0-100 (StatValue bounds)

#### Health Mechanics

Egg health creates tension and risk/reward gameplay:

```rust
impl Pet {
    /// Update egg health based on warmth level
    /// Called every tick
    pub fn update_egg_health(&mut self, delta: Duration) {
        if self.stage != LifeStage::Egg || self.egg_stats.is_dead() {
            return;
        }
        
        // Health damage thresholds
        const CRITICAL_WARMTH: u8 = 20;   // Below this, health drops fast
        const LOW_WARMTH: u8 = 30;        // Below this, health drops slowly
        
        let health_loss = if self.egg_stats.warmth_level < CRITICAL_WARMTH {
            // Critical cold: -5 health per second
            5.0 * delta.as_secs_f32()
        } else if self.egg_stats.warmth_level < LOW_WARMTH {
            // Low warmth: -2 health per second
            2.0 * delta.as_secs_f32()
        } else {
            // Comfortable: no health loss, slight recovery
            -0.5 * delta.as_secs_f32()  // Negative = recovery
        };
        
        if health_loss > 0.0 {
            self.egg_stats.health = 
                self.egg_stats.health.saturating_sub(health_loss as u8);
        } else {
            self.egg_stats.health = 
                (self.egg_stats.health + (-health_loss as u8)).min(100);
        }
        
        // Check for game over
        if self.egg_stats.health == 0 {
            self.trigger_game_over();
        }
    }
    
    fn trigger_game_over(&mut self) {
        self.state = PetState::Dead { at: Utc::now() };
        // Game over state will be handled by the game loop
    }
}
```

**Health Mechanics:**
- **Visible condition**: Health only appears in UI when `warmth < 30`
- **Safe zone**: Warmth ≥ 30 = no health loss, slight recovery
- **Danger zone**: Warmth 20-30 = -2 health per second
- **Critical zone**: Warmth < 20 = -5 health per second
- **Recovery**: Warmth ≥ 30 = +0.5 health per second
- **Game Over**: Health reaches 0 = permadeath

#### Context-Aware Status Messages

The egg provides feedback based on its current state:

```rust
impl Pet {
    /// Get a status message describing the egg's state
    pub fn get_egg_status_message(&self) -> String {
        if self.stage != LifeStage::Egg {
            return "The pet has hatched!".to_string();
        }
        
        let warmth = self.egg_stats.warmth_level;
        let progress = self.egg_stats.incubation_progress;
        let health = self.egg_stats.health;
        
        // Priority 1: Critical health/warmth warnings
        if self.egg_stats.health < 30 {
            return format!(
                "⚠️  CRITICAL: The egg is dying! Health: {}% - Warm it NOW!",
                health
            );
        }
        
        if warmth < 20 {
            return "🧊 The egg is freezing! Press [W] to warm it!".to_string();
        }
        
        // Priority 2: Incubation progress
        if progress >= 90.0 {
            return "🎉 The egg is wiggling! It's about to hatch!".to_string();
        }
        
        if progress >= 70.0 {
            return "🥚 The egg is wobbling... Getting close!".to_string();
        }
        
        if progress >= 50.0 {
            return "🌡️  The egg is warm and developing nicely.".to_string();
        }
        
        // Priority 3: Warmth-based comfort
        if warmth >= 70 {
            return format!(
                "😊 The egg is cozy and warm! ({:.0}% incubated)",
                progress
            );
        }
        
        if warmth >= 50 {
            return format!(
                "😐 The egg feels comfortable. ({:.0}% incubated)",
                progress
            );
        }
        
        if warmth >= 30 {
            return format!(
                "🙂 The egg is cool but okay. ({:.0}% incubated)",
                progress
            );
        }
        
        // Warmth 20-30 (low but not critical)
        format!(
            "⚠️  The egg feels cold... ({:.0}% incubated, Health visible)",
            progress
        )
    }
}
```

**Message Thresholds:**

| Warmth | Progress | Message |
|--------|----------|---------|
| < 20 | Any | "🧊 The egg is freezing! Press [W] to warm it!" |
| 20-30 | Any | "⚠️ The egg feels cold... (X% incubated, Health visible)" |
| 30-50 | Any | "🙂 The egg is cool but okay. (X% incubated)" |
| 50-70 | Any | "😐 The egg feels comfortable. (X% incubated)" |
| 70+ | Any | "😊 The egg is cozy and warm! (X% incubated)" |
| Any | 50%+ | "🌡️ The egg is warm and developing nicely." |
| Any | 70%+ | "🥚 The egg is wobbling... Getting close!" |
| Any | 90%+ | "🎉 The egg is wiggling! It's about to hatch!" |
| < 30 HP | < 30 | "⚠️ CRITICAL: The egg is dying! Health: X% - Warm it NOW!" |

#### Hatching Mechanics

When incubation reaches 100%, the egg hatches:

```rust
impl Pet {
    /// Hatch the egg into a baby
    /// Called automatically when incubation_progress >= 100%
    pub fn hatch(&mut self) {
        if self.stage != LifeStage::Egg {
            return;
        }
        
        // Advance to baby stage
        self.stage = LifeStage::Baby;
        
        // Calculate stat bonuses based on egg care
        let stat_bonus = self.calculate_hatching_bonus();
        
        // Apply bonus to starting stats
        self.stats.hunger = StatValue::new(50 + stat_bonus.hunger_bonus);
        self.stats.happiness = StatValue::new(70 + stat_bonus.happiness_bonus);
        self.stats.energy = StatValue::new(80 + stat_bonus.energy_bonus);
        self.stats.health = StatValue::new(100);  // Always full health
        self.stats.hygiene = StatValue::new(80 + stat_bonus.hygiene_bonus);
        
        // Transition to normal state
        self.state = PetState::Normal;
        
        // Clear egg stats (no longer needed)
        // In save format, egg_stats can be omitted or kept for history
    }
    
    /// Calculate stat bonuses based on how well the egg was cared for
    fn calculate_hatching_bonus(&self) -> HatchingBonus {
        let mut bonus = HatchingBonus::default();
        
        // Bonus based on minimum health during incubation
        // (Track min_health during egg phase)
        let health_ratio = self.egg_stats.health as f32 / 100.0;
        
        if health_ratio > 0.9 {
            // Perfect care: +10 to all stats
            bonus.hunger_bonus = 10;
            bonus.happiness_bonus = 10;
            bonus.energy_bonus = 10;
            bonus.hygiene_bonus = 10;
            bonus.description = "Perfectly incubated! The baby is thriving!";
        } else if health_ratio > 0.7 {
            // Good care: +5 to happiness and energy
            bonus.happiness_bonus = 5;
            bonus.energy_bonus = 5;
            bonus.description = "Well cared for. The baby is healthy and happy.";
        } else if health_ratio > 0.5 {
            // Okay care: +3 to happiness only
            bonus.happiness_bonus = 3;
            bonus.description = "Adequate care. The baby seems content.";
        } else {
            // Poor care: no bonus, slight penalty
            bonus.hunger_bonus = -5;
            bonus.description = "The egg was neglected. The baby seems weak.";
        }
        
        bonus
    }
}

/// Stat bonuses applied when hatching
#[derive(Debug, Clone, Copy, Default)]
pub struct HatchingBonus {
    pub hunger_bonus: i8,
    pub happiness_bonus: i8,
    pub energy_bonus: i8,
    pub hygiene_bonus: i8,
    pub description: &'static str,
}
```

**Hatching Bonuses:**

| Final Health | Hunger | Happiness | Energy | Hygiene | Description |
|--------------|--------|-----------|--------|---------|-------------|
| 90-100% | +10 | +10 | +10 | +10 | Perfectly incubated! The baby is thriving! |
| 70-89% | 0 | +5 | +5 | 0 | Well cared for. The baby is healthy and happy. |
| 50-69% | 0 | +3 | 0 | 0 | Adequate care. The baby seems content. |
| < 50% | -5 | 0 | 0 | 0 | The egg was neglected. The baby seems weak. |

#### UI Changes for Egg Stage

The stats panel changes to show egg-specific information:

```rust
// In src/ui.rs - draw_egg_stats function

fn draw_egg_stats(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let egg_stats = &app.pet.egg_stats;
    
    let mut text: Vec<Line> = vec![
        Line::from(vec![
            Span::raw("Stage: "),
            Span::styled("Egg", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        // Incubation progress bar
        Line::from(format!(
            "Incubation: {}", 
            stat_bar(egg_stats.incubation_progress as u8)
        )),
        // Warmth level (color-coded)
        Line::from(vec![
            Span::raw("Warmth:     "),
            Span::styled(
                stat_bar(egg_stats.warmth_level),
                warmth_style(egg_stats.warmth_level)
            ),
        ]),
    ];
    
    // Only show health if warmth is low
    if egg_stats.should_show_health() {
        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::raw("Health:     "),
            Span::styled(
                stat_bar(egg_stats.health),
                health_style(egg_stats.health)
            ),
        ]));
    }
    
    // Status message
    text.push(Line::from(""));
    text.push(Line::from(app.pet.get_egg_status_message()));
    
    // Controls hint
    text.push(Line::from(""));
    text.push(Line::from(
        "[W] Warm the egg"
    ).style(Style::default().fg(Color::Yellow)));
    
    let stats_widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Egg Status"));
    
    frame.render_widget(stats_widget, area);
}

/// Get style based on warmth level
fn warmth_style(warmth: u8) -> Style {
    match warmth {
        0..20 => Style::default().fg(Color::Red),
        20..40 => Style::default().fg(Color::Yellow),
        40..70 => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::Cyan),
    }
}

/// Get style based on health level
fn health_style(health: u8) -> Style {
    match health {
        0..30 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        30..60 => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Green),
    }
}
```

**Egg Stage UI Features:**
- **Incubation bar**: Shows percentage to hatching
- **Warmth bar**: Color-coded (red/yellow/green/cyan)
- **Health bar**: Hidden unless warmth < 30 (creates tension)
- **Status message**: Context-aware feedback
- **Control hint**: "[W] Warm the egg" always visible

### Baby Stage Restrictions

Baby pets have special restrictions to make them more realistic and challenging:

```rust
impl Pet {
    /// Play with the pet (with Baby restrictions)
    pub fn play(&mut self) -> Result<(), &'static str> {
        if self.stage == LifeStage::Egg {
            return Err("Can't play with an egg!");
        }

        if !self.state.can_act() {
            return Err("Pet cannot play right now");
        }

        // Baby stage restrictions
        if self.stage == LifeStage::Baby {
            if self.stats.energy.value() < 30 {
                return Err("Baby is too tired. Let it sleep first!");
            }
            // Baby can't play for long
            self.stats.happiness.add(15); // Less happiness gain
            self.stats.energy.sub(20);    // More energy cost
            self.stats.hunger.sub(10);
            return Ok(());
        }

        // Normal play for other stages...
    }
}
```

**Baby Stage Rules:**
- Higher energy requirement for play (30 instead of 20)
- Less happiness gain from playing (15 instead of 20)
- More energy cost from playing (20 instead of 15)
- **Sleep bonuses**: Babies recover 50% more energy from sleep
- UI shows "[P]lay (Gentle)" to indicate baby restrictions

## Step 3: Pet State Machine

Add to `src/pet.rs`:

```rust
use chrono::{DateTime, Utc};

/// Pet behavior states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PetState {
    /// Normal active state
    Normal,
    /// Sleeping (regenerates energy)
    Sleeping { since: DateTime<Utc> },
    /// Sick (loses health over time)
    Sick { since: DateTime<Utc> },
    /// Dead (terminal state)
    Dead { at: DateTime<Utc> },
}

impl PetState {
    /// Check if pet can perform an action
    pub fn can(&self, action: Action) -> bool {
        match self {
            PetState::Normal => true,
            PetState::Sleeping { .. } => {
                matches!(action, Action::Wake)
            }
            PetState::Sick { .. } => {
                // Can do most actions but not play
                !matches!(action, Action::Play)
            }
            PetState::Dead { .. } => false,
        }
    }
    
    /// Check if transition is valid
    pub fn can_transition_to(&self, new_state: &PetState) -> bool {
        use PetState::*;
        
        match (self, new_state) {
            // Dead is terminal
            (Dead { .. }, _) => false,
            
            // Can wake up from sleep
            (Sleeping { .. }, Normal) => true,
            
            // Can recover from sickness
            (Sick { .. }, Normal) => true,
            
            // Can transition from normal to anything
            (Normal, _) => true,
            
            // Can't transition to same state
            (a, b) if std::mem::discriminant(a) == std::mem::discriminant(b) => false,
            
            _ => false,
        }
    }
    
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            PetState::Normal => "Awake",
            PetState::Sleeping { .. } => "Sleeping",
            PetState::Sick { .. } => "Sick",
            PetState::Dead { .. } => "Dead",
        }
    }
}

impl Default for PetState {
    fn default() -> Self {
        PetState::Normal
    }
}

/// Actions the player can take
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Feed,
    Play,
    Clean,
    Sleep,
    Wake,
    Medicine,
}
```

## Step 4: Pet Struct

Add to `src/pet.rs`:

```rust
use crate::stats::{StatValue, Stats};

/// The virtual pet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    /// Pet name
    pub name: String,
    
    /// Current life stage
    pub stage: LifeStage,
    
    /// Current behavior state
    pub state: PetState,
    
    /// Current stats
    pub stats: Stats,
    
    /// When the pet was born
    pub birth_time: DateTime<Utc>,
    
    /// Last time player interacted
    pub last_interaction: DateTime<Utc>,
    
    /// Total seconds lived (for age calculation)
    pub age_seconds: u64,
}

impl Pet {
    /// Create a new pet (starts as egg)
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        
        Self {
            name: name.into(),
            stage: LifeStage::Egg,
            state: PetState::Normal,
            stats: Stats::egg(),
            birth_time: now,
            last_interaction: now,
            age_seconds: 0,
        }
    }
    
    /// Get current age
    pub fn age(&self) -> Duration {
        Duration::from_secs(self.age_seconds)
    }
    
    /// Update age and check for stage progression
    pub fn update_age(&mut self, delta: Duration) -> Option<LifeStage> {
        let old_stage = self.stage;
        
        self.age_seconds += delta.as_secs();
        let current_age = self.age();
        
        // Check if we should advance stage
        while let Some(next) = self.stage.next() {
            if current_age >= next.threshold() {
                self.stage = next;
            } else {
                break;
            }
        }
        
        if self.stage != old_stage {
            Some(self.stage)
        } else {
            None
        }
    }
    
    /// Check if pet is alive
    pub fn is_alive(&self) -> bool {
        !matches!(self.state, PetState::Dead { .. })
    }
    
    /// Update last interaction time
    pub fn interacted(&mut self) {
        self.last_interaction = Utc::now();
    }
    
    /// Get status summary
    pub fn status_summary(&self) -> String {
        if !self.is_alive() {
            return format!("{} has passed away.", self.name);
        }
        
        let (worst_stat, value) = self.stats.worst_stat();
        
        if value < 25 {
            format!("{} needs attention! {} is critical.", self.name, worst_stat)
        } else if value < 50 {
            format!("{} is okay, but {} could be better.", self.name, worst_stat)
        } else {
            format!("{} is doing well!", self.name)
        }
    }
}

impl Default for Pet {
    fn default() -> Self {
        Self::new("Fluffy")
    }
}
```

## Step 5: ASCII Art Loading

Create `src/assets.rs`:

```rust
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::pet::LifeStage;
use crate::error::Result;

/// ASCII art cache
pub struct AsciiArtCache {
    cache: HashMap<LifeStage, Vec<String>>,
}

impl AsciiArtCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();
        
        // Load default art for each stage
        cache.insert(LifeStage::Egg, Self::egg_art());
        cache.insert(LifeStage::Baby, Self::baby_art());
        cache.insert(LifeStage::Child, Self::child_art());
        cache.insert(LifeStage::Teen, Self::teen_art());
        cache.insert(LifeStage::Adult, Self::adult_art());
        
        Self { cache }
    }
    
    pub fn get(&self, stage: LifeStage) -> &[String] {
        self.cache.get(&stage)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
    
    /// Placeholder art - replace with real files later
    fn egg_art() -> Vec<String> {
        vec![
            "   _~^~^~_   ".to_string(),
            ")\\_\   /_/(  ".to_string(),
            "  / _x_ \\   ".to_string(),
            "  \\_____/   ".to_string(),
        ]
    }
    
    fn baby_art() -> Vec<String> {
        vec![
            "   ^~^  /".to_string(),
            "  (O O) ".to_string(),
            "  ( > ) ".to_string(),
            "--m-m---".to_string(),
        ]
    }
    
    fn child_art() -> Vec<String> {
        vec![
            "    /\\_/\\    ".to_string(),
            "   ( o.o )   ".to_string(),
            "    > ^ <    ".to_string(),
            "   /|   |\\   ".to_string(),
            "  (_|   |_)  ".to_string(),
        ]
    }
    
    fn teen_art() -> Vec<String> {
        vec![
            "      /\\_____/\\     ".to_string(),
            "     /  o   o  \\    ".to_string(),
            "    ( ==  ^  == )    ".to_string(),
            "     )         (     ".to_string(),
            "    (           )    ".to_string(),
            "   ( (  )   (  ) )   ".to_string(),
            "  (__(__)___(__)__)  ".to_string(),
        ]
    }
    
    fn adult_art() -> Vec<String> {
        vec![
            "       /\\     /\\       ".to_string(),
            "      /  \\   /  \\      ".to_string(),
            "     (    \\ /    )     ".to_string(),
            "     |  o   o   |      ".to_string(),
            "     |   ___)   |      ".to_string(),
            "     \\  (____   /      ".to_string(),
            "      (_____   /       ".to_string(),
            "      /         \\      ".to_string(),
        ]
    }
}

impl Default for AsciiArtCache {
    fn default() -> Self {
        Self::new()
    }
}
```

## Step 6: Update App to Include Pet

Modify `src/app.rs`:

```rust
use crate::pet::Pet;
use crate::assets::AsciiArtCache;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub tick_count: u64,
    pub pet: Pet,
    pub art_cache: AsciiArtCache,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            tick_count: 0,
            pet: Pet::new("Fluffy"),
            art_cache: AsciiArtCache::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn tick(&mut self) {
        self.tick_count += 1;
        
        // Update pet age every 10 ticks (1 second at 100ms tick rate)
        if self.tick_count % 10 == 0 {
            use std::time::Duration;
            if let Some(new_stage) = self.pet.update_age(Duration::from_secs(1)) {
                // Log stage change (will implement proper logging later)
                println!("Pet evolved to {:?}!", new_stage);
            }
        }
    }
    
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent
) {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('r') => {
                // Reset pet (for testing)
                self.pet = Pet::new("Fluffy");
            }
            _ => {}
        }
    }
}
```

## Step 7: Update UI to Display Pet

Modify `src/ui.rs`:

```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Clear},
    Frame,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.size();
    
    // Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(10),     // Main content
            Constraint::Length(3),   // Footer
        ])
        .split(size);
    
    // Header
    let header = Paragraph::new(format!("MyPet TUI - {}", app.pet.name))
        .style(Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);
    
    // Main content split
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    // Pet display
    draw_pet(frame, app, main_chunks[0]);
    
    // Stats display
    draw_stats(frame, app, main_chunks[1]);
    
    // Footer
    let footer = Paragraph::new("[q]uit | [r]eset | Age will increase automatically")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn draw_pet(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let art = app.art_cache.get(app.pet.stage);
    
    let art_text = Text::from(
        art.iter()
            .map(|line| Line::from(line.clone()))
            .collect::<Vec<_>>()
    );
    
    let pet_widget = Paragraph::new(art_text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("{} ({:?})", app.pet.stage.name(), app.pet.state.name())));
    
    frame.render_widget(pet_widget, area);
}

fn draw_stats(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let stats = &app.pet.stats;
    
    let text = vec![
        Line::from(vec![
            Span::raw("Stage: "),
            Span::styled(app.pet.stage.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(format!("Hunger:    {}", stat_bar(stats.hunger.value()))),
        Line::from(format!("Happiness: {}", stat_bar(stats.happiness.value()))),
        Line::from(format!("Energy:    {}", stat_bar(stats.energy.value()))),
        Line::from(format!("Health:    {}", stat_bar(stats.health.value()))),
        Line::from(format!("Hygiene:   {}", stat_bar(stats.hygiene.value()))),
        Line::from(""),
        Line::from(format!("Age: {} seconds", app.pet.age_seconds)),
        Line::from(format!("Status: {}", app.pet.state.name())),
    ];
    
    let stats_widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Stats"));
    
    frame.render_widget(stats_widget, area);
}

fn stat_bar(value: u8) -> String {
    let filled = (value as usize * 20) / 100;
    let empty = 20 - filled;
    
    format!("[{}{}] {:3}%",
        "█".repeat(filled),
        "░".repeat(empty),
        value
    )
}
```

## Step 8: Update Main

Update `src/main.rs` to include new modules:

```rust
mod app;
mod assets;
mod error;
mod event;
mod pet;
mod stats;
mod tui;
mod ui;

// ... rest of main.rs remains same
```

## Phase 2 Success Criteria

- [x] Project compiles with all new modules
- [x] Pet struct with all stats displays correctly
- [x] Stats show visual bars (█ and ░ characters)
- [x] Pet automatically ages (age counter increases)
- [x] Pet ASCII art changes with life stage
- [x] Life stage transitions happen at thresholds
- [x] Pet state (Normal/Sleeping/Sick/Dead) is displayed
- [x] StatValue correctly clamps to 0-100 range

## Next Steps

Move to **Phase 3: Interactions** to implement feeding, playing, and other actions.

## Testing Commands

```bash
# Run and wait to see aging
cargo run

# Watch the pet age through stages
# Press 'r' to reset
# Press 'q' to quit
```
