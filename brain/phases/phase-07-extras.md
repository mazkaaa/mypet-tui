# Phase 7: Extras

**Status**: 📋 Not Started  
**Duration**: Week 4+  
**Goal**: Mini-games, achievements, multiple species, and polish

## Overview

This phase adds features that make the game complete and fun: mini-games during play, an achievement system, multiple pet species with different traits, and configuration options.

## Prerequisites

- Phase 6 complete (save/load working)
- All core gameplay implemented

## Step 1: Mini-Game System

Create `src/minigames/mod.rs`:

```rust
pub mod catch;
pub mod memory;

use crate::error::Result;
use crate::pet::Pet;

/// Result of playing a mini-game
#[derive(Debug, Clone)]
pub struct MiniGameResult {
    pub success: bool,
    pub score: u32,
    pub happiness_reward: u8,
    pub energy_cost: u8,
    pub message: String,
}

/// Mini-game trait
pub trait MiniGame {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn play(&self, pet: &Pet) -> MiniGameResult;
}

/// Available mini-games
#[derive(Debug, Clone, Copy)]
pub enum MiniGameType {
    Catch,   // Catch falling objects
    Memory,  // Simon-says style
}

impl MiniGameType {
    pub fn get_game(&self) -> Box<dyn MiniGame> {
        match self {
            MiniGameType::Catch => Box::new(catch::CatchGame::new()),
            MiniGameType::Memory => Box::new(memory::MemoryGame::new()),
        }
    }
    
    pub fn all() -> [MiniGameType; 2] {
        [MiniGameType::Catch, MiniGameType::Memory]
    }
}
```

### Catch Mini-Game

Create `src/minigames/catch.rs`:

```rust
use super::{MiniGame, MiniGameResult};
use crate::pet::Pet;

pub struct CatchGame;

impl CatchGame {
    pub fn new() -> Self {
        Self
    }
}

impl MiniGame for CatchGame {
    fn name(&self) -> &str {
        "Catch the Treat"
    }
    
    fn description(&self) -> &str {
        "Catch falling treats by moving left/right!"
    }
    
    fn play(&self, pet: &Pet) -> MiniGameResult {
        // Simplified implementation
        // In real game, this would be interactive
        
        let score = fastrand::u32(0..100);
        let success = score > 50;
        
        let happiness = if success { 25 } else { 10 };
        let message = if success {
            format!("Great job! {} caught {} treats!", pet.name, score / 10)
        } else {
            format!("Good try! {} caught {} treats.", pet.name, score / 10)
        };
        
        MiniGameResult {
            success,
            score,
            happiness_reward: happiness,
            energy_cost: 15,
            message,
        }
    }
}
```

### Memory Mini-Game

Create `src/minigames/memory.rs`:

```rust
use super::{MiniGame, MiniGameResult};
use crate::pet::Pet;

pub struct MemoryGame;

impl MemoryGame {
    pub fn new() -> Self {
        Self
    }
}

impl MiniGame for MemoryGame {
    fn name(&self) -> &str {
        "Pattern Memory"
    }
    
    fn description(&self) -> &str {
        "Remember and repeat the pattern!"
    }
    
    fn play(&self, pet: &Pet) -> MiniGameResult {
        let rounds = fastrand::u32(3..8);
        let score = rounds * 10;
        
        MiniGameResult {
            success: rounds >= 5,
            score,
            happiness_reward: (rounds * 3) as u8,
            energy_cost: 10,
            message: format!("{} remembered {} patterns!", pet.name, rounds),
        }
    }
}
```

## Step 2: Achievement System

Create `src/achievements.rs`:

```rust
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Achievement ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    FirstFeed,
    FirstPlay,
    FirstClean,
    FirstMedicine,
    EvolutionBaby,
    EvolutionChild,
    EvolutionTeen,
    EvolutionAdult,
    PerfectCare,      // All stats > 90
    LongLife,         // Pet lived 1 hour
    MiniGameMaster,   // Won 10 mini-games
    NoDeath,          // Reached adult without sickness
}

impl AchievementId {
    pub fn name(&self) -> &str {
        match self {
            AchievementId::FirstFeed => "First Meal",
            AchievementId::FirstPlay => "Playtime",
            AchievementId::FirstClean => "Sparkling Clean",
            AchievementId::FirstMedicine => "Dr. Pet",
            AchievementId::EvolutionBaby => "Welcome to the World",
            AchievementId::EvolutionChild => "Growing Up",
            AchievementId::EvolutionTeen => "Teen Spirit",
            AchievementId::EvolutionAdult => "Full Grown",
            AchievementId::PerfectCare => "Perfect Parent",
            AchievementId::LongLife => "Lifetime Companion",
            AchievementId::MiniGameMaster => "Game Master",
            AchievementId::NoDeath => "Immortal",
        }
    }
    
    pub fn description(&self) -> &str {
        match self {
            AchievementId::FirstFeed => "Feed your pet for the first time",
            AchievementId::FirstPlay => "Play with your pet",
            AchievementId::FirstClean => "Clean your pet",
            AchievementId::FirstMedicine => "Heal your pet with medicine",
            AchievementId::EvolutionBaby => "Your pet hatched from egg",
            AchievementId::EvolutionChild => "Your pet became a child",
            AchievementId::EvolutionTeen => "Your pet became a teen",
            AchievementId::EvolutionAdult => "Your pet reached adulthood",
            AchievementId::PerfectCare => "Keep all stats above 90",
            AchievementId::LongLife => "Keep your pet alive for 1 hour",
            AchievementId::MiniGameMaster => "Win 10 mini-games",
            AchievementId::NoDeath => "Reach adult without getting sick",
        }
    }
}

/// Achievement tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Achievements {
    unlocked: HashSet<AchievementId>,
    mini_games_won: u32,
}

impl Achievements {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Try to unlock an achievement
    pub fn unlock(&mut self, id: AchievementId) -> bool {
        if self.unlocked.insert(id) {
            log::info!("Achievement unlocked: {:?}", id);
            true
        } else {
            false
        }
    }
    
    /// Check if achievement is unlocked
    pub fn has(&self, id: AchievementId) -> bool {
        self.unlocked.contains(&id)
    }
    
    /// Get all unlocked achievements
    pub fn unlocked_list(&self) -> Vec<AchievementId> {
        self.unlocked.iter().copied().collect()
    }
    
    /// Get count of unlocked achievements
    pub fn count(&self) -> usize {
        self.unlocked.len()
    }
    
    /// Total number of achievements
    pub fn total() -> usize {
        12 // Update as achievements are added
    }
    
    /// Record mini-game win
    pub fn record_win(&mut self) {
        self.mini_games_won += 1;
        
        if self.mini_games_won >= 10 {
            self.unlock(AchievementId::MiniGameMaster);
        }
    }
    
    /// Check stat-based achievements
    pub fn check_stats(&mut self, stats: &crate::stats::Stats) {
        if stats.hunger.value() < 10
            && stats.happiness.value() > 90
            && stats.energy.value() > 90
            && stats.health.value() > 90
            && stats.hygiene.value() > 90
        {
            self.unlock(AchievementId::PerfectCare);
        }
    }
}
```

## Step 3: Multiple Species

Create `src/species.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::stats::Stats;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Species {
    Cat,
    Dog,
    Rabbit,
    Bird,
    Dragon,
}

impl Species {
    pub fn name(&self) -> &str {
        match self {
            Species::Cat => "Cat",
            Species::Dog => "Dog",
            Species::Rabbit => "Rabbit",
            Species::Bird => "Bird",
            Species::Dragon => "Dragon",
        }
    }
    
    /// Starting stats modifier
    pub fn starting_stats(&self) -> Stats {
        let mut stats = Stats::new();
        
        match self {
            Species::Cat => {
                // Balanced
                stats.happiness = 70.into();
            }
            Species::Dog => {
                // High energy, needs more play
                stats.energy = 90.into();
                stats.happiness = 60.into();
            }
            Species::Rabbit => {
                // Clean but fragile
                stats.hygiene = 90.into();
                stats.health = 80.into();
            }
            Species::Bird => {
                // High happiness, fragile
                stats.happiness = 80.into();
                stats.health = 70.into();
            }
            Species::Dragon => {
                // Rare, slow aging, high stats
                stats.hunger = 30.into();
                stats.happiness = 80.into();
                stats.energy = 90.into();
                stats.health = 100.into();
                stats.hygiene = 80.into();
            }
        }
        
        stats
    }
    
    /// Stat decay modifiers (multipliers)
    pub fn decay_rates(&self) -> DecayModifiers {
        match self {
            Species::Cat => DecayModifiers::default(),
            Species::Dog => DecayModifiers {
                happiness: 1.5,  // Needs more attention
                energy: 1.2,
                ..Default::default()
            },
            Species::Rabbit => DecayModifiers {
                hygiene: 0.5,    // Stays cleaner
                health: 1.3,     // But fragile
                ..Default::default()
            },
            Species::Bird => DecayModifiers {
                happiness: 1.3,
                energy: 0.8,
                ..Default::default()
            },
            Species::Dragon => DecayModifiers {
                hunger: 0.5,
                happiness: 0.7,
                energy: 0.6,
                hygiene: 0.8,
                health: 0.4,
            },
        }
    }
    
    /// ASCII art prefix for this species
    pub fn art_prefix(&self) -> &str {
        match self {
            Species::Cat => "cat",
            Species::Dog => "dog",
            Species::Rabbit => "rabbit",
            Species::Bird => "bird",
            Species::Dragon => "dragon",
        }
    }
    
    /// All available species
    pub fn all() -> [Species; 5] {
        [
            Species::Cat,
            Species::Dog,
            Species::Rabbit,
            Species::Bird,
            Species::Dragon,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct DecayModifiers {
    pub hunger: f32,
    pub happiness: f32,
    pub energy: f32,
    pub hygiene: f32,
    pub health: f32,
}

impl Default for DecayModifiers {
    fn default() -> Self {
        Self {
            hunger: 1.0,
            happiness: 1.0,
            energy: 1.0,
            hygiene: 1.0,
            health: 1.0,
        }
    }
}
```

## Step 4: Configuration System

Create `src/config.rs`:

```rust
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::species::Species;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Pet name (if None, prompt on start)
    pub default_pet_name: Option<String>,
    
    /// Default species
    pub default_species: Species,
    
    /// Time scale (1.0 = real time, 60.0 = 1 sec = 1 minute)
    pub time_scale: f32,
    
    /// Enable auto-save
    pub auto_save: bool,
    
    /// Auto-save interval in seconds
    pub auto_save_interval_secs: u64,
    
    /// Show notifications
    pub notifications: bool,
    
    /// UI theme
    pub theme: String,
    
    /// Pet art style (ascii, unicode, minimal)
    pub art_style: ArtStyle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ArtStyle {
    Minimal,  // Simple (◕‿◕)
    Ascii,    // Full ASCII art
    Unicode,  // With unicode box drawing
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_pet_name: None,
            default_species: Species::Cat,
            time_scale: 60.0,
            auto_save: true,
            auto_save_interval_secs: 60,
            notifications: true,
            theme: "default".to_string(),
            art_style: ArtStyle::Ascii,
        }
    }
}

impl Config {
    /// Load config from file or create default
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        
        if path.exists() {
            let toml = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&toml)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let toml = toml::to_string_pretty(self)?;
        fs::write(path, toml)?;
        
        Ok(())
    }
    
    /// Get config file path
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| crate::error::AppError::Save(
                "Could not find config directory".to_string()
            ))?;
        
        Ok(config_dir.join("mypet-tui").join("config.toml"))
    }
}
```

## Step 5: Name Selection on Start

Create `src/ui/name_select.rs`:

```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct NameSelect {
    input: String,
    cursor: usize,
}

impl NameSelect {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
        }
    }
    
    pub fn handle_input(&mut self, c: char) {
        if c.is_ascii_alphanumeric() || c == ' ' || c == '-' {
            self.input.insert(self.cursor, c);
            self.cursor += 1;
        }
    }
    
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.input.remove(self.cursor);
        }
    }
    
    pub fn confirm(&self) -> Option<String> {
        if self.input.is_empty() {
            None
        } else {
            Some(self.input.clone())
        }
    }
    
    pub fn draw(&self, frame: &mut Frame) {
        let area = centered_rect(60, 20, frame.size());
        
        frame.render_widget(Clear, area);
        
        let block = Block::default()
            .title("Name Your Pet")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        
        let text = vec![
            Line::from("What would you like to name your pet?"),
            Line::from(""),
            Line::from(vec![
                Span::raw("Name: "),
                Span::styled(&self.input, Style::default().fg(Color::Yellow)),
                Span::styled("_", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from("Press Enter to confirm, Esc to cancel"),
        ];
        
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(block);
        
        frame.render_widget(paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

## Step 6: Species Selection

Create `src/ui/species_select.rs`:

```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::species::Species;

pub struct SpeciesSelect {
    species: Vec<Species>,
    selected: usize,
}

impl SpeciesSelect {
    pub fn new() -> Self {
        Self {
            species: Species::all().to_vec(),
            selected: 0,
        }
    }
    
    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.species.len();
    }
    
    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.species.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
    
    pub fn selected(&self) -> Species {
        self.species[self.selected]
    }
    
    pub fn draw(&self, frame: &mut Frame) {
        let area = centered_rect(70, 40, frame.size());
        
        frame.render_widget(Clear, area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(area);
        
        // Title
        let title = Paragraph::new("Choose Your Pet")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        frame.render_widget(title, chunks[0]);
        
        // Species list
        let items: Vec<ListItem> = self.species.iter().enumerate()
            .map(|(i, s)| {
                let style = if i == self.selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                ListItem::new(s.name()).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(list, chunks[1]);
    }
}

// Reuse centered_rect from name_select
```

## Phase 7 Success Criteria

- [x] At least 2 mini-games implemented
- [x] Mini-game results affect pet stats
- [x] Achievement system tracks progress
- [x] Achievements unlock during gameplay
- [x] Multiple species with different traits
- [x] Species affects starting stats and decay rates
- [x] Configuration file saves user preferences
- [x] Name selection on first start
- [x] Species selection available
- [x] Time scale adjustable for testing/gameplay

## Polish Checklist

Additional items to consider:

- [ ] Sound effects (optional feature)
- [ ] Desktop notifications when pet needs care
- [ ] Easter eggs (special animations, secret species)
- [ ] Screenshots of pet
- [ ] Export/import save files
- [ ] Multiple save slots
- [ ] Pet diary/log of life events
- [ ] Statistics tracking (total play time, actions taken)

## Final Testing

```bash
# Full test run
cargo test

# Run the complete game
cargo run --release

# Test all features:
# - Create new pet with name
# - Select species
# - Play mini-games
# - Check achievements
# - Verify save/load
```

## Completion

Congratulations! You've built a complete virtual pet game in the terminal.

## Resources for Further Development

- [Ratatui Examples](https://github.com/ratatui/ratatui/tree/main/examples)
- [Crossterm Event Handling](https://docs.rs/crossterm/latest/crossterm/event/index.html)
- [Serde Documentation](https://serde.rs/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
