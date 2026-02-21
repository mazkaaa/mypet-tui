# Rust Tips for MyPet TUI

## Common Patterns

### 1. Bounded Integer Types

Always validate bounds, use newtypes to prevent invalid values:

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct StatValue(u8);

impl StatValue {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 100;
    
    pub fn new(value: u8) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }
    
    pub fn add(&mut self, delta: i16) {
        let new = (self.0 as i16 + delta).clamp(0, 100) as u8;
        self.0 = new;
    }
    
    pub fn value(&self) -> u8 { self.0 }
    pub fn is_critical(&self) -> bool { self.0 < 25 }
}
```

### 2. Type-Safe State Machines

Use enums with data instead of bool flags:

```rust
// Good
pub enum PetState {
    Normal,
    Sleeping { since: DateTime<Utc> },
    Sick { since: DateTime<Utc>, severity: u8 },
    Dead { at: DateTime<Utc> },
}

// Bad
pub struct PetState {
    is_sleeping: bool,
    is_sick: bool,
    is_dead: bool,
    sleep_start: Option<DateTime<Utc>>,
    sick_start: Option<DateTime<Utc>>,
}
```

### 3. Duration Calculations

Use `Duration` for all time calculations:

```rust
use std::time::Duration;

pub const TICK_RATE: Duration = Duration::from_millis(100);
pub const SAVE_INTERVAL: Duration = Duration::from_secs(30);
pub const AGE_THRESHOLD_BABY: Duration = Duration::from_secs(60);

// Calculate age
let age = birth_time.elapsed();
if age >= AGE_THRESHOLD_BABY {
    evolve_to(LifeStage::Baby);
}
```

### 4. Configuration with Default

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_tick_rate")]
    pub tick_rate_ms: u64,
    
    #[serde(default)]
    pub pet_name: Option<String>,
    
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate_ms: default_tick_rate(),
            pet_name: None,
            theme: default_theme(),
        }
    }
}

fn default_tick_rate() -> u64 { 100 }
fn default_theme() -> String { "default".to_string() }
```

## Error Handling

### 1. Custom Error Type

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid action: {0} cannot {1}")]
    InvalidAction(PetState, Action),
    
    #[error("Stat out of bounds: {0}")]
    StatOutOfBounds(&'static str),
    
    #[error("Save file corrupted")]
    CorruptedSave,
}

pub type Result<T> = std::result::Result<T, PetError>;
```

### 2. Graceful Degradation

```rust
impl App {
    pub fn load_save(&mut self) -> Result<()> {
        match SaveData::load() {
            Ok(data) => {
                self.pet = data.pet;
                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to load save: {}", e);
                // Start fresh instead of crashing
                self.pet = Pet::default();
                self.add_log("Welcome! Starting a new pet.");
                Ok(())
            }
        }
    }
}
```

## Collections

### 1. Circular Buffer for Event Log

```rust
use std::collections::VecDeque;

pub struct EventLog {
    events: VecDeque<String>,
    capacity: usize,
}

impl EventLog {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fn push(&mut self, event: impl Into<String>) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event.into());
    }
    
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &String> {
        self.events.iter().rev().take(n).rev()
    }
}
```

### 2. Enum Map Pattern

```rust
use std::collections::HashMap;

pub struct StatModifiers {
    modifiers: HashMap<StatType, Vec<Modifier>>,
}

impl StatModifiers {
    pub fn add(&mut self, stat: StatType, modifier: Modifier) {
        self.modifiers.entry(stat).or_default().push(modifier);
    }
    
    pub fn apply(&self, stat: StatType, base_value: u8) -> u8 {
        let modifiers = self.modifiers.get(&stat).map(|v| v.as_slice()).unwrap_or(&[]);
        modifiers.iter().fold(base_value, |acc, m| m.apply(acc))
    }
}
```

## Async Patterns

### 1. Shared State with Arc<Mutex<_>>

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct GameLoop {
    app: Arc<Mutex<App>>,
}

impl GameLoop {
    pub fn new(app: App) -> Self {
        Self {
            app: Arc::new(Mutex::new(app)),
        }
    }
    
    pub async fn run(&self) {
        let app = Arc::clone(&self.app);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                let mut app = app.lock().await;
                app.tick();
            }
        });
    }
}
```

### 2. Select for Multiple Event Sources

```rust
use tokio::select;

async fn run(&mut self) -> Result<()> {
    let mut tick = tokio::time::interval(TICK_RATE);
    let mut events = event::EventStream::new();
    
    loop {
        select! {
            _ = tick.tick() => {
                self.on_tick();
            }
            
            Some(Ok(event)) = events.next() => {
                if !self.handle_event(event) {
                    break;
                }
            }
            
            // Save trigger channel
            Some(_) = self.save_rx.recv() => {
                self.save()?;
            }
        }
        
        self.draw().await?;
    }
    
    Ok(())
}
```

## Serialization

### 1. Custom Serde for Complex Types

```rust
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug, Clone)]
pub struct DurationWrapper(Duration);

impl Serialize for DurationWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer 
    {
        // Serialize as seconds
        serializer.serialize_u64(self.0.as_secs())
    }
}

impl<'de> Deserialize<'de> for DurationWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> 
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(DurationWrapper(Duration::from_secs(secs)))
    }
}
```

### 2. Versioned Save Format

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum SaveData {
    #[serde(rename = "1")]
    V1(SaveDataV1),
    #[serde(rename = "2")]
    V2(SaveDataV2),
}

impl SaveData {
    pub fn to_current(self) -> SaveDataV2 {
        match self {
            SaveData::V1(v1) => v1.migrate(),
            SaveData::V2(v2) => v2,
        }
    }
}
```

## Testing

### 1. Time-Based Tests with Mock Clock

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stat_decay() {
        let mut pet = Pet::new("Test");
        pet.stats.hunger = StatValue::new(50);
        
        // Simulate 10 seconds passing
        pet.update(Duration::from_secs(10));
        
        // Hunger should increase (get worse) by ~5 points
        assert!(pet.stats.hunger.value() > 50);
        assert!(pet.stats.hunger.value() <= 55);
    }
}
```

### 2. Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn stat_value_never_exceeds_bounds(value: u8, delta: i16) {
        let mut stat = StatValue::new(value);
        stat.add(delta);
        
        prop_assert!(stat.value() <= 100);
        prop_assert!(stat.value() >= 0);
    }
}
```

## Performance

### 1. Avoid String Allocations in Hot Path

```rust
// Bad - allocates every frame
fn render(&self) {
    let text = format!("Hunger: {}/100", self.hunger);
    // ...
}

// Good - use stack buffer
fn render(&self, buf: &mut [u8; 32]) {
    use std::io::Write;
    write!(&mut buf[..], "Hunger: {}/100", self.hunger).unwrap();
    // ...
}
```

### 2. Lazy Static for Constants

```rust
use std::sync::LazyLock;

pub static DEFAULT_PET_NAMES: LazyLock<Vec<String>> = 
    LazyLock::new(|| {
        vec![
            "Fluffy".to_string(),
            "Buddy".to_string(),
            "Max".to_string(),
            "Luna".to_string(),
        ]
    });
```

## Debugging

### 1. Structured Logging

```rust
use tracing::{info, warn, error, debug, span, Level};

fn feed_pet(&mut self, amount: u8) {
    let span = span!(Level::INFO, "feed", amount);
    let _enter = span.enter();
    
    debug!(pet = %self.pet.name, "Feeding pet");
    
    let old_hunger = self.pet.stats.hunger.value();
    self.pet.stats.hunger.subtract(amount);
    
    info!(
        old_hunger,
        new_hunger = self.pet.stats.hunger.value(),
        "Pet fed successfully"
    );
}
```

### 2. Debug Impl for Complex Types

```rust
impl fmt::Debug for Pet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pet")
            .field("name", &self.name)
            .field("stage", &self.stage)
            .field("hunger", &self.stats.hunger.value())
            .field("happiness", &self.stats.happiness.value())
            // Skip internal fields
            .finish()
    }
}
```
