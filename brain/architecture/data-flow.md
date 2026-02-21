# Data Flow Architecture

## Overview

This document describes the data flow through the MyPet TUI application, from user input to screen rendering.

## Architecture Pattern: Event-Driven with Centralized State

We use an event-driven architecture with a centralized state store (the `App` struct). All state mutations flow through a single update path.

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Input     │────▶│   Event     │────▶│    State    │────▶│   Render    │
│  (Keyboard) │     │  Handling   │     │   Update    │     │   (Ratatui) │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
       │                                    ▲                      │
       │                                    │                      │
       └────────────────────────────────────┘◄─────────────────────┘
                                    (Game Loop Tick)
```

## Data Flow Stages

### 1. Input Capture (Crossterm)

**Location**: `src/tui.rs`

```rust
// Crossterm reads raw terminal input
if let Event::Key(key) = event::read()? {
    // Convert to our Action enum
    let action = match key.code {
        KeyCode::Char('f') => Some(Action::Feed),
        KeyCode::Char('q') => Some(Action::Quit),
        // ... etc
    };
}
```

**Key Points**:
- Uses `crossterm::event::read()` for blocking input
- Non-blocking with `poll()` for game loop integration
- Raw mode enables instant key detection (no Enter required)

### 2. Event Processing

**Location**: `src/app.rs` - `handle_event()`

Input events are converted to `Action` variants and queued for processing:

```rust
pub fn handle_event(&mut self, event: Event) -> Result<Option<Action>> {
    match event {
        Event::Key(key_event) => self.handle_key_event(key_event),
        Event::Tick => self.handle_tick_event(),  // Game loop timer
        _ => Ok(None),
    }
}
```

### 3. State Updates

**Location**: `src/app.rs` - `update()`

All state mutations happen in the update method:

```rust
pub fn update(&mut self, action: Action) {
    match action {
        Action::Feed => self.handle_feed(),
        Action::Play => self.handle_play(),
        Action::Tick => self.handle_game_tick(),  // Time-based updates
        // ... etc
    }
}
```

**Update Order** (important for consistency):
1. Process user actions
2. Update pet state machine
3. Update animation system
4. Update game time/stat decay
5. Check win/lose conditions

### 4. Animation System Flow

**Location**: `src/animation.rs`

```
User Action ──▶ AnimationRequest ──▶ AnimationQueue ──▶ ActiveAnimation
                                                          │
                                                          ▼
                                                    Frame Updates (timer)
                                                          │
                                                          ▼
                                                    Render with current frame
```

```rust
pub struct AnimationEngine {
    queue: VecDeque<AnimationRequest>,
    current: Option<ActiveAnimation>,
    last_update: Instant,
}

impl AnimationEngine {
    pub fn update(&mut self, now: Instant) {
        if let Some(ref mut anim) = self.current {
            if anim.should_advance_frame(now) {
                anim.advance_frame();
            }
            
            if anim.is_complete() {
                self.current = self.queue.pop_front();
            }
        }
    }
}
```

### 5. Rendering Pipeline

**Location**: `src/ui.rs` - `draw()`

```rust
pub fn draw(&mut self, frame: &mut Frame) {
    // 1. Clear and setup layout
    let layout = self.calculate_layout(frame.size());
    
    // 2. Draw pet (with current animation frame)
    self.draw_pet(frame, layout.pet_area);
    
    // 3. Draw stats
    self.draw_stats(frame, layout.stats_area);
    
    // 4. Draw UI chrome
    self.draw_ui_chrome(frame, layout.ui_area);
    
    // 5. Draw event log
    self.draw_event_log(frame, layout.log_area);
}
```

**Render Order** (back to front):
1. Background/color fills
2. Static UI elements (borders, labels)
3. Pet display (ASCII art)
4. Particles/effects overlays
5. Foreground UI (popups, notifications)

## Key Data Structures

### Message/Event Types

```rust
// User-initiated actions
pub enum Action {
    Feed,
    Play,
    Clean,
    Sleep,
    Medicine,
    Quit,
    Tick,  // Game loop timer event
}

// Internal state change events
pub enum GameEvent {
    StatChanged { stat: StatType, old: u8, new: u8 },
    StageChanged { from: LifeStage, to: LifeStage },
    StateChanged { from: PetState, to: PetState },
    PetDied,
    RandomEvent(EventType),
}
```

### State Container

```rust
pub struct App {
    // Core state
    pub pet: Pet,
    pub running: bool,
    
    // Systems
    pub animation: AnimationEngine,
    pub event_log: VecDeque<String>,
    
    // Timing
    pub last_tick: Instant,
    pub tick_rate: Duration,
    
    // UI State
    pub current_screen: Screen,
    pub popup: Option<Popup>,
}
```

## Async/Timer Flow (Tokio)

**Location**: `src/game_loop.rs`

```rust
pub async fn run_game_loop(app: Arc<Mutex<App>>) {
    let mut tick_interval = interval(Duration::from_millis(100));
    let mut animation_interval = interval(Duration::from_millis(100)); // 10 FPS
    
    loop {
        tokio::select! {
            // Game tick (stat decay, aging)
            _ = tick_interval.tick() => {
                let mut app = app.lock().await;
                app.update(Action::Tick);
            }
            
            // Animation frame update
            _ = animation_interval.tick() => {
                let mut app = app.lock().await;
                app.animation.update(Instant::now());
            }
        }
    }
}
```

## Thread Safety

Since Ratatui requires single-threaded access to the terminal, we use:
- `Arc<Mutex<App>>` for shared state between game loop and main thread
- All terminal operations happen in the main thread
- Game loop sends render requests via message passing

## Data Persistence Flow

```
Auto-save Timer ──▶ Serialize App State ──▶ Write to Disk
                              │
                              ▼
                       JSON format (human-readable)
```

```rust
// Save flow
impl App {
    pub fn save(&self) -> Result<()> {
        let save_data = SaveData::from(self);
        let json = serde_json::to_string_pretty(&save_data)?;
        fs::write(self.save_path(), json)?;
        Ok(())
    }
}
```

## Performance Considerations

1. **Minimize allocations in render loop**: Pre-allocate string buffers, reuse Vecs
2. **Dirty checking**: Only re-render changed regions (Ratatui handles this automatically)
3. **Animation frame caching**: Load animation frames once, store in memory
4. **Event batching**: Process multiple events between renders if input is rapid

## Error Handling Strategy

All fallible operations return `Result<T, AppError>`:

```rust
pub enum AppError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Terminal(ratatui::crossterm::ErrorKind),
}

// Critical errors: Log and exit gracefully
// Recoverable errors: Show in event log, continue
```
