# Phase 1: Foundation

**Status**: 📋 Not Started  
**Duration**: Week 1  
**Goal**: Working terminal app with basic Ratatui integration

## Overview

This phase establishes the project foundation: Cargo project setup, Ratatui integration, event handling, and terminal management. By the end, you should have a working terminal window that can display content and respond to keyboard input.

## Prerequisites

- Rust toolchain installed (rustup)
- Basic Rust knowledge
- Terminal emulator with Unicode support

## Step 1: Project Setup

### 1.1 Create Cargo Project

```bash
cd /home/mazka/development/mypet-tui
cargo init --name mypet-tui
cd mypet-tui
```

### 1.2 Configure Cargo.toml

```toml
[package]
name = "mypet-tui"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A terminal-based virtual pet game"

[dependencies]
# Core TUI framework
ratatui = "0.30.0"

# Terminal backend
crossterm = "0.28.0"

# Error handling
thiserror = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# For later phases
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
dirs = "6.0"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
# Testing utilities
```

Run `cargo check` to download dependencies.

### 1.3 Project Structure

Create the directory structure:

```bash
mkdir -p src/widgets src/assets
```

Initial file structure:
```
src/
├── main.rs          # Entry point
├── app.rs           # App state and main loop
├── tui.rs           # Terminal setup/cleanup
├── ui.rs            # UI rendering
├── error.rs         # Error types
└── event.rs         # Event handling
```

## Step 2: Error Handling

Create `src/error.rs`:

```rust
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Terminal error: {0}")]
    Terminal(String),
    
    #[error("Event handling error: {0}")]
    Event(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

## Step 3: Terminal Management (tui.rs)

Create `src/tui.rs`:

```rust
use std::io::{self, stdout, Stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use crate::error::Result;

/// Type alias for our terminal backend
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal
/// 
/// Sets up:
/// - Raw mode (input immediately available, no Enter needed)
/// - Alternate screen buffer (doesn't scroll main terminal)
/// - Mouse capture (optional)
pub fn init() -> Result<Tui> {
    // Enable raw mode
    enable_raw_mode()?;
    
    // Setup stdout with alternate screen
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableMouseCapture)?;
    
    // Create terminal
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    
    Ok(terminal)
}

/// Restore terminal to original state
/// 
/// CRITICAL: Always call this before exiting, or terminal will be broken
pub fn restore(terminal: &mut Tui) -> Result<()> {
    // Disable mouse capture
    terminal.backend_mut()
        .execute(DisableMouseCapture)?;
    
    // Leave alternate screen
    terminal.backend_mut()
        .execute(LeaveAlternateScreen)?;
    
    // Disable raw mode
    disable_raw_mode()?;
    
    Ok(())
}
```

### Decision: Raw Mode vs Alternate Screen

**Raw Mode**:
- Input available immediately (no Enter needed)
- Disables line buffering
- Disables echo (keys don't show on screen)
- **Required** for real-time games

**Alternate Screen Buffer**:
- Terminal content preserved when app exits
- No scrollback pollution
- Clean slate for drawing
- **Required** for professional TUI apps

**Both are essential** for MyPet TUI.

## Step 4: App State (app.rs)

Create `src/app.rs`:

```rust
use crate::error::Result;

#[derive(Debug)]
pub struct App {
    /// Is the app running?
    pub running: bool,
    
    /// Current tick count
    pub tick_count: u64,
    
    /// Test message (for this phase)
    pub message: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            tick_count: 0,
            message: "Welcome to MyPet TUI! Press 'q' to quit.".to_string(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Handle a tick event (called on timer)
    pub fn tick(&mut self) {
        self.tick_count += 1;
    }
    
    /// Handle keyboard input
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Char('q') => {
                self.running = false;
            }
            KeyCode::Char(c) => {
                self.message = format!("You pressed: {}", c);
            }
            _ => {}
        }
    }
    
    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}
```

## Step 5: Event Handling (event.rs)

Create `src/event.rs`:

```rust
use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyEvent};

use crate::error::{AppError, Result};

/// Application events
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// Timer tick
    Tick,
    /// Keyboard input
    Key(KeyEvent),
}

/// Event handler with non-blocking poll
pub struct EventHandler {
    /// Tick rate (how often Tick events fire)
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }
    
    /// Poll for next event
    /// 
    /// Returns immediately if event available, otherwise waits up to tick_rate
    pub fn next(&self) -> Result<Event> {
        // Check if event available (non-blocking with timeout)
        if event::poll(self.tick_rate)? {
            // Read the event
            if let CEvent::Key(key) = event::read()? {
                return Ok(Event::Key(key));
            }
        }
        
        // No keyboard event, return tick
        Ok(Event::Tick)
    }
}
```

## Step 6: UI Rendering (ui.rs)

Create `src/ui.rs`:

```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

/// Draw the UI
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.size();
    
    // Clear the screen
    frame.render_widget(Clear, size);
    
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(size);
    
    // Header
    let header = Paragraph::new("MyPet TUI - v0.1.0")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);
    
    // Main content
    let text = vec![
        Line::from(app.message.clone()),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tick count: ", Style::default()),
            Span::styled(
                app.tick_count.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];
    
    let main = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Pet Display"))
        .wrap(Wrap { trim: true });
    frame.render_widget(main, chunks[1]);
    
    // Footer
    let footer = Paragraph::new("Controls: [q]uit | Press any key to test")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}
```

## Step 7: Main Entry Point (main.rs)

Create `src/main.rs`:

```rust
mod app;
mod error;
mod event;
mod tui;
mod ui;

use std::time::Duration;

use app::App;
use error::Result;
use event::{Event, EventHandler};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create app state
    let mut app = App::new();
    
    // Setup terminal
    let mut terminal = tui::init()?;
    
    // Create event handler (100ms tick rate)
    let events = EventHandler::new(Duration::from_millis(100));
    
    // Main loop
    let result = run_app(&mut app, &mut terminal, &events);
    
    // CRITICAL: Always restore terminal, even if app panics
    tui::restore(&mut terminal)?;
    
    // Handle any errors from main loop
    result?;
    
    Ok(())
}

fn run_app(
    app: &mut App,
    terminal: &mut tui::Tui,
    events: &EventHandler,
) -> Result<()> {
    while app.running {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;
        
        // Handle events
        match events.next()? {
            Event::Tick => {
                app.tick();
            }
            Event::Key(key) => {
                app.handle_key(key);
            }
        }
    }
    
    Ok(())
}
```

## Step 8: Build and Test

```bash
# Check for compilation errors
cargo check

# Build the project
cargo build

# Run the app
cargo run
```

Expected behavior:
- Terminal clears and shows the MyPet TUI interface
- Press keys to see them displayed
- Tick count increments automatically
- Press 'q' to exit cleanly
- Terminal returns to normal state

## Common Issues and Solutions

### Issue: Terminal not restored on panic

**Solution**: Use panic hook:

```rust
fn main() {
    // Setup panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Restore terminal
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = std::io::stdout()
            .execute(crossterm::terminal::LeaveAlternateScreen);
        
        original_hook(info);
    }));
    
    // ... rest of main
}
```

### Issue: Events not responding

**Solution**: Check raw mode is enabled:

```rust
// Verify raw mode
assert!(crossterm::terminal::is_raw_mode_enabled()?);
```

### Issue: Flickering display

**Solution**: Don't clear screen every frame:

```rust
// Remove this:
// frame.render_widget(Clear, size);

// Ratatui handles clearing automatically
```

## Phase 1 Success Criteria

- [x] Project compiles without warnings
- [x] `cargo run` opens a terminal UI
- [x] UI displays header, content area, and footer
- [x] Keyboard input is detected immediately (no Enter needed)
- [x] Pressing 'q' exits the application
- [x] Terminal is restored to normal state after exit
- [x] Tick counter increments automatically

## Next Steps

Once this phase is complete, move to **Phase 2: Pet System** to implement the core pet data structures and stats.

## Resources

- [Ratatui Hello World Tutorial](https://ratatui.rs/tutorials/hello-world/)
- [Crossterm Documentation](https://docs.rs/crossterm/latest/crossterm/)
- [Terminal Raw Mode Explained](https://ratatui.rs/concepts/backends/)
