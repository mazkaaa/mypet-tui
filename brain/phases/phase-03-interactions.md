# Phase 3: Interactions

**Status**: 📋 Not Started  
**Duration**: Week 2  
**Goal**: Implement all player actions with validation and effects

## Overview

This phase implements the interaction system: feeding, playing, cleaning, sleeping, and medicine. Each action has validation (can you feed a sleeping pet?), immediate and delayed effects, and triggers appropriate animations.

## Prerequisites

- Phase 2 complete (Pet struct with stats)
- Understanding of the Command pattern
- Event system for triggering animations

## Step 1: Action System with Command Pattern

Create `src/actions.rs`:

```rust
use crate::error::Result;
use crate::pet::{Action, Pet, PetState};
use crate::stats::StatValue;
use chrono::Utc;

/// Result of executing an action
#[derive(Debug, Clone)]
pub struct ActionResult {
    /// Whether the action succeeded
    pub success: bool,
    /// Message to display to player
    pub message: String,
    /// Stat changes that occurred
    pub stat_changes: Vec<StatChange>,
    /// Animation to trigger
    pub animation: Option<AnimationTrigger>,
    /// Sound effect to play (future)
    pub sound: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StatChange {
    pub stat: StatType,
    pub old_value: u8,
    pub new_value: u8,
    pub delta: i16,
}

#[derive(Debug, Clone, Copy)]
pub enum StatType {
    Hunger,
    Happiness,
    Energy,
    Health,
    Hygiene,
}

#[derive(Debug, Clone, Copy)]
pub enum AnimationTrigger {
    Eating,
    Playing,
    Cleaning,
    Sleeping,
    Waking,
    Medicine,
    Happy,
    Sad,
}

/// Execute an action on a pet
pub fn execute_action(pet: &mut Pet, action: Action) -> ActionResult {
    // First check if action is valid in current state
    if !pet.state.can(action) {
        return ActionResult {
            success: false,
            message: format!("Cannot {:?} while {}", action, pet.state.name()),
            stat_changes: vec![],
            animation: None,
            sound: None,
        };
    }
    
    match action {
        Action::Feed => feed(pet),
        Action::Play => play(pet),
        Action::Clean => clean(pet),
        Action::Sleep => sleep(pet),
        Action::Wake => wake(pet),
        Action::Medicine => medicine(pet),
    }
}

fn feed(pet: &mut Pet) -> ActionResult {
    let mut changes = vec![];
    
    // Can't feed if full
    if pet.stats.hunger.value() <= 10 {
        return ActionResult {
            success: false,
            message: format!("{} is not hungry!", pet.name),
            stat_changes: changes,
            animation: Some(AnimationTrigger::Sad),
            sound: None,
        };
    }
    
    // Apply hunger reduction
    let old_hunger = pet.stats.hunger.value();
    pet.stats.hunger.add(-30);
    changes.push(StatChange {
        stat: StatType::Hunger,
        old_value: old_hunger,
        new_value: pet.stats.hunger.value(),
        delta: -(old_hunger as i16 - pet.stats.hunger.value() as i16),
    });
    
    // Feeding costs a little energy (digestion)
    let old_energy = pet.stats.energy.value();
    pet.stats.energy.add(-5);
    changes.push(StatChange {
        stat: StatType::Energy,
        old_value: old_energy,
        new_value: pet.stats.energy.value(),
        delta: -(old_energy as i16 - pet.stats.energy.value() as i16),
    });
    
    // Slight happiness boost
    let old_happy = pet.stats.happiness.value();
    pet.stats.happiness.add(5);
    changes.push(StatChange {
        stat: StatType::Happiness,
        old_value: old_happy,
        new_value: pet.stats.happiness.value(),
        delta: pet.stats.happiness.value() as i16 - old_happy as i16,
    });
    
    pet.interacted();
    
    ActionResult {
        success: true,
        message: format!("You fed {}! Yum!", pet.name),
        stat_changes: changes,
        animation: Some(AnimationTrigger::Eating),
        sound: Some("eat".to_string()),
    }
}

fn play(pet: &mut Pet) -> ActionResult {
    let mut changes = vec![];
    
    // Need energy to play
    if pet.stats.energy.value() < 30 {
        return ActionResult {
            success: false,
            message: format!("{} is too tired to play!", pet.name),
            stat_changes: changes,
            animation: Some(AnimationTrigger::Sad),
            sound: None,
        };
    }
    
    // Big happiness boost
    let old_happy = pet.stats.happiness.value();
    pet.stats.happiness.add(25);
    changes.push(StatChange {
        stat: StatType::Happiness,
        old_value: old_happy,
        new_value: pet.stats.happiness.value(),
        delta: pet.stats.happiness.value() as i16 - old_happy as i16,
    });
    
    // Costs energy
    let old_energy = pet.stats.energy.value();
    pet.stats.energy.add(-20);
    changes.push(StatChange {
        stat: StatType::Energy,
        old_value: old_energy,
        new_value: pet.stats.energy.value(),
        delta: -(old_energy as i16 - pet.stats.energy.value() as i16),
    });
    
    // Makes pet hungry
    let old_hunger = pet.stats.hunger.value();
    pet.stats.hunger.add(10);
    changes.push(StatChange {
        stat: StatType::Hunger,
        old_value: old_hunger,
        new_value: pet.stats.hunger.value(),
        delta: pet.stats.hunger.value() as i16 - old_hunger as i16,
    });
    
    // Gets a bit dirty
    let old_hygiene = pet.stats.hygiene.value();
    pet.stats.hygiene.add(-10);
    changes.push(StatChange {
        stat: StatType::Hygiene,
        old_value: old_hygiene,
        new_value: pet.stats.hygiene.value(),
        delta: -(old_hygiene as i16 - pet.stats.hygiene.value() as i16),
    });
    
    pet.interacted();
    
    ActionResult {
        success: true,
        message: format!("You played with {}! So much fun!", pet.name),
        stat_changes: changes,
        animation: Some(AnimationTrigger::Playing),
        sound: Some("play".to_string()),
    }
}

fn clean(pet: &mut Pet) -> ActionResult {
    let mut changes = vec![];
    
    // Restore hygiene
    let old_hygiene = pet.stats.hygiene.value();
    pet.stats.hygiene = StatValue::new(100);
    changes.push(StatChange {
        stat: StatType::Hygiene,
        old_value: old_hygiene,
        new_value: 100,
        delta: 100 - old_hygiene as i16,
    });
    
    // Slight happiness boost if was dirty
    let old_happy = pet.stats.happiness.value();
    if old_hygiene < 50 {
        pet.stats.happiness.add(10);
        changes.push(StatChange {
            stat: StatType::Happiness,
            old_value: old_happy,
            new_value: pet.stats.happiness.value(),
            delta: pet.stats.happiness.value() as i16 - old_happy as i16,
        });
    }
    
    pet.interacted();
    
    ActionResult {
        success: true,
        message: format!("You cleaned {}! Sparkling!", pet.name),
        stat_changes: changes,
        animation: Some(AnimationTrigger::Cleaning),
        sound: Some("clean".to_string()),
    }
}

fn sleep(pet: &mut Pet) -> ActionResult {
    let mut changes = vec![];
    
    // Check if already sleeping
    if matches!(pet.state, PetState::Sleeping { .. }) {
        return ActionResult {
            success: false,
            message: format!("{} is already sleeping!", pet.name),
            stat_changes: changes,
            animation: None,
            sound: None,
        };
    }
    
    // Change state
    let old_state = pet.state.clone();
    pet.state = PetState::Sleeping { since: Utc::now() };
    
    pet.interacted();
    
    ActionResult {
        success: true,
        message: format!("{} went to sleep. Sweet dreams!", pet.name),
        stat_changes: changes,
        animation: Some(AnimationTrigger::Sleeping),
        sound: Some("sleep".to_string()),
    }
}

fn wake(pet: &mut Pet) -> ActionResult {
    let mut changes = vec![];
    
    // Check if sleeping
    if !matches!(pet.state, PetState::Sleeping { .. }) {
        return ActionResult {
            success: false,
            message: format!("{} is not sleeping!", pet.name),
            stat_changes: changes,
            animation: None,
            sound: None,
        };
    }
    
    pet.state = PetState::Normal;
    pet.interacted();
    
    ActionResult {
        success: true,
        message: format!("{} woke up! Good morning!", pet.name),
        stat_changes: changes,
        animation: Some(AnimationTrigger::Waking),
        sound: Some("wake".to_string()),
    }
}

fn medicine(pet: &mut Pet) -> ActionResult {
    let mut changes = vec![];
    
    // Heal if sick
    if matches!(pet.state, PetState::Sick { .. }) {
        pet.state = PetState::Normal;
        
        // Restore some health
        let old_health = pet.stats.health.value();
        pet.stats.health.add(20);
        changes.push(StatChange {
            stat: StatType::Health,
            old_value: old_health,
            new_value: pet.stats.health.value(),
            delta: pet.stats.health.value() as i16 - old_health as i16,
        });
        
        pet.interacted();
        
        return ActionResult {
            success: true,
            message: format!("You gave {} medicine. They feel better!", pet.name),
            stat_changes: changes,
            animation: Some(AnimationTrigger::Medicine),
            sound: Some("heal".to_string()),
        };
    }
    
    // Can still give medicine to healthy pet (waste)
    let old_health = pet.stats.health.value();
    if old_health < 100 {
        pet.stats.health.add(10);
        changes.push(StatChange {
            stat: StatType::Health,
            old_value: old_health,
            new_value: pet.stats.health.value(),
            delta: pet.stats.health.value() as i16 - old_health as i16,
        });
        
        pet.interacted();
        
        ActionResult {
            success: true,
            message: format!("You gave {} medicine.", pet.name),
            stat_changes: changes,
            animation: None,
            sound: None,
        }
    } else {
        ActionResult {
            success: false,
            message: format!("{} doesn't need medicine!", pet.name),
            stat_changes: changes,
            animation: Some(AnimationTrigger::Sad),
            sound: None,
        }
    }
}

impl Action {
    /// Get the keyboard key for this action
    pub fn key(&self) -> char {
        match self {
            Action::Feed => 'f',
            Action::Play => 'p',
            Action::Clean => 'c',
            Action::Sleep => 's',
            Action::Wake => 'w',
            Action::Medicine => 'm',
        }
    }
    
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Action::Feed => "Feed",
            Action::Play => "Play",
            Action::Clean => "Clean",
            Action::Sleep => "Sleep",
            Action::Wake => "Wake",
            Action::Medicine => "Medicine",
        }
    }
    
    /// Get all available actions
    pub fn all() -> [Action; 6] {
        [
            Action::Feed,
            Action::Play,
            Action::Clean,
            Action::Sleep,
            Action::Wake,
            Action::Medicine,
        ]
    }
}
```

## Step 2: Event Log System

Create `src/event_log.rs`:

```rust
use std::collections::VecDeque;

/// Event log for displaying history
#[derive(Debug, Clone)]
pub struct EventLog {
    events: VecDeque<LogEntry>,
    max_size: usize,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub message: String,
    pub category: LogCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCategory {
    Info,
    Success,
    Warning,
    Error,
}

impl EventLog {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn add(&mut self, message: impl Into<String>, category: LogCategory) {
        let entry = LogEntry {
            timestamp: format!("{:02}:{:02}", 
                chrono::Local::now().hour(),
                chrono::Local::now().minute()
            ),
            message: message.into(),
            category,
        };
        
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        
        self.events.push_back(entry);
    }
    
    pub fn info(&mut self, message: impl Into<String>) {
        self.add(message, LogCategory::Info);
    }
    
    pub fn success(&mut self, message: impl Into<String>) {
        self.add(message, LogCategory::Success);
    }
    
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(message, LogCategory::Warning);
    }
    
    pub fn error(&mut self, message: impl Into<String>) {
        self.add(message, LogCategory::Error);
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.events.iter()
    }
    
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &LogEntry> {
        self.events.iter().rev().take(n).rev()
    }
    
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new(100)
    }
}
```

## Step 3: Update App with Actions

Update `src/app.rs`:

```rust
use crate::actions::{execute_action, ActionResult, AnimationTrigger};
use crate::event_log::EventLog;
use crate::pet::Action;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub tick_count: u64,
    pub pet: Pet,
    pub art_cache: AsciiArtCache,
    pub event_log: EventLog,
    pub current_animation: Option<AnimationTrigger>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            tick_count: 0,
            pet: Pet::new("Fluffy"),
            art_cache: AsciiArtCache::new(),
            event_log: EventLog::new(50),
            current_animation: None,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn handle_action(&mut self, action: Action) {
        let result = execute_action(&mut self.pet, action);
        
        // Log the result
        if result.success {
            self.event_log.success(result.message);
        } else {
            self.event_log.warning(result.message);
        }
        
        // Trigger animation
        if let Some(anim) = result.animation {
            self.current_animation = Some(anim);
        }
        
        // Log stat changes (for feedback)
        for change in result.stat_changes {
            let direction = if change.delta > 0 { "↑" } else { "↓" };
            self.event_log.info(format!(
                "{:?}: {} {} ({} → {})",
                change.stat, direction, change.delta.abs(),
                change.old_value, change.new_value
            ));
        }
    }
    
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('f') => self.handle_action(Action::Feed),
            KeyCode::Char('p') => self.handle_action(Action::Play),
            KeyCode::Char('c') => self.handle_action(Action::Clean),
            KeyCode::Char('s') => {
                // Sleep or wake depending on current state
                if matches!(self.pet.state, crate::pet::PetState::Sleeping { .. }) {
                    self.handle_action(Action::Wake);
                } else {
                    self.handle_action(Action::Sleep);
                }
            }
            KeyCode::Char('m') => self.handle_action(Action::Medicine),
            KeyCode::Char('r') => {
                self.pet = Pet::new("Fluffy");
                self.event_log.info("Pet reset!");
            }
            _ => {}
        }
    }
    
    // ... tick() remains same
}
```

## Step 4: Update UI

Add to `src/ui.rs`:

```rust
fn draw_event_log(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::style::{Color, Style};
    
    let lines: Vec<Line> = app.event_log
        .recent(5)
        .map(|entry| {
            let color = match entry.category {
                LogCategory::Info => Color::Gray,
                LogCategory::Success => Color::Green,
                LogCategory::Warning => Color::Yellow,
                LogCategory::Error => Color::Red,
            };
            
            Line::from(vec![
                Span::styled(format!("[{}] ", entry.timestamp), Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.message, Style::default().fg(color)),
            ])
        })
        .collect();
    
    let log_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Event Log"));
    
    frame.render_widget(log_widget, area);
}

fn draw_actions(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::style::{Color, Style};
    
    let mut action_text = vec![];
    
    for action in Action::all() {
        let available = app.pet.state.can(action);
        let color = if available { Color::White } else { Color::DarkGray };
        
        action_text.push(Span::styled(
            format!("[{}]{}", action.key(), action.display_name()),
            Style::default().fg(color),
        ));
        action_text.push(Span::raw("  "));
    }
    
    action_text.push(Span::styled("[r]eset  ", Style::default()));
    action_text.push(Span::styled("[q]uit", Style::default()));
    
    let actions_widget = Paragraph::new(Line::from(action_text))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Actions"));
    
    frame.render_widget(actions_widget, area);
}
```

Update main draw function:

```rust
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.size();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(10),     // Main (pet + stats)
            Constraint::Length(3),   // Actions
            Constraint::Length(8),   // Event log
        ])
        .split(size);
    
    // Header
    let header = Paragraph::new(format!("MyPet TUI - {}", app.pet.name))
        .style(Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);
    
    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    draw_pet(frame, app, main_chunks[0]);
    draw_stats(frame, app, main_chunks[1]);
    
    // Actions
    draw_actions(frame, app, chunks[2]);
    
    // Event log
    draw_event_log(frame, app, chunks[3]);
}
```

## Phase 3 Success Criteria

- [x] All actions implemented with proper validation
- [x] Actions check state before executing (can't feed sleeping pet)
- [x] Stat changes are logged and displayed
- [x] Event log shows history of actions
- [x] Keyboard shortcuts work for all actions
- [x] Action availability shown in UI (dimmed if unavailable)
- [x] Appropriate error messages for invalid actions
- [x] Animation triggers are set (actual animations in Phase 5)

## Testing

```bash
cargo run

# Test interactions:
# f - feed (should reduce hunger)
# p - play (should increase happiness, reduce energy)
# c - clean (should set hygiene to 100)
# s - sleep (pet state changes to sleeping)
# s - wake (while sleeping, wakes up)
# m - medicine (try when sick)
```

## Next Steps

Move to **Phase 4: Game Loop** for real-time stat decay and random events.
