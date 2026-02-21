# AGENTS.md - MyPet TUI

## Project Overview

MyPet TUI is a terminal-based virtual pet game inspired by Tamagotchi, built with Rust and Ratatui.

- **Language**: Rust (Edition 2024)
- **TUI Framework**: Ratatui (v0.30.0+) with Crossterm backend
- **Async Runtime**: Tokio
- **Serialization**: serde + serde_json

## Build Commands

```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run the application
cargo run

# Check compilation without building
cargo check

# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run tests with output visible
cargo test -- --nocapture

# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Run clippy with all features
cargo clippy --all-features -- -D warnings
```

## Code Style Guidelines

### Imports

- Group imports: std, external crates, internal modules (separated by blank lines)
- Use absolute paths for internal modules (`crate::`)
- Re-export commonly used types at module level

```rust
use std::time::Duration;

use tokio::time::interval;
use serde::{Serialize, Deserialize};

use crate::pet::{Pet, PetState};
```

### Types and Naming

- Use `PascalCase` for types, structs, enums, traits
- Use `snake_case` for functions, variables, modules
- Use `SCREAMING_SNAKE_CASE` for constants
- Use `newtype` pattern for bounded values (e.g., `StatValue(u8)`)
- Prefer type-safe state machines over boolean flags

```rust
// Good: enum with data
pub enum PetState {
    Normal,
    Sleeping { since: DateTime<Utc> },
    Sick { since: DateTime<Utc>, severity: u8 },
}

// Bad: boolean flags
pub struct PetState {
    is_sleeping: bool,
    is_sick: bool,
}
```

### Error Handling

- Use `thiserror` for custom error types
- Provide `#[from]` implementations for wrapped errors
- Use `Result<T>` type alias for consistency
- Graceful degradation over panics for recoverable errors

```rust
#[derive(Error, Debug)]
pub enum PetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid action: {0} cannot {1}")]
    InvalidAction(PetState, Action),
}

pub type Result<T> = std::result::Result<T, PetError>;
```

### State Management

- Use `Duration` for all time calculations
- Implement `Default` for configuration structs
- Separate game state from UI state
- Use command pattern for actions that modify state
- Validate invariants after loading saves

### Async Patterns

- Use `Arc<Mutex<_>>` for shared state across tasks
- Use `tokio::select!` for multiple event sources
- Prefer `tokio::sync::Mutex` over `std::sync::Mutex` in async contexts

### Testing

- Write unit tests in `#[cfg(test)]` modules
- Use property-based testing for bounded values
- Mock time for time-based tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_value_never_exceeds_bounds() {
        let mut stat = StatValue::new(50);
        stat.add(100);
        assert_eq!(stat.value(), 100);
    }
}
```

### Documentation

- Document public APIs with rustdoc comments
- Include examples in doc comments where helpful
- Use `//!` for module-level documentation
- Always see the documentation, workplan, and knowledge base first before implementing

## Project Structure

```
src/
├── main.rs          # Entry point, CLI args
├── app.rs           # App state and main loop
├── tui.rs           # Terminal setup and event handling
├── pet.rs           # Pet struct and logic
├── stats.rs         # Stats system
├── actions.rs       # Player actions
├── ui.rs            # UI rendering
├── animation.rs     # Animation engine
├── game_loop.rs     # Time-based updates
├── save.rs          # Save/load functionality
└── widgets/         # Custom Ratatui widgets
    ├── mod.rs
    ├── pet_display.rs
    ├── stat_bar.rs
    └── event_log.rs
```

## Dependencies

Key crates used in this project:

- `ratatui` - TUI framework
- `crossterm` - Terminal backend
- `tokio` - Async runtime
- `serde` + `serde_json` - Serialization
- `thiserror` - Error handling
- `tracing` - Structured logging
- `chrono` - Date/time handling

## Architecture Notes

- **Immediate mode rendering** via Ratatui
- **Hierarchical state ownership**: App -> Pet -> Stats/State
- **Command pattern** for user actions
- **Time-based decay** for pet stats
- **Versioned save format** for backward compatibility

## References

- See `brain/` directory for detailed architecture docs
- See `WORKPLAN.md` for feature roadmap

## Token Efficiency

- Never re-read files you just wrote or edited. You know the contents.
- Never re-run commands to "verify" unless the outcome was uncertain.
- Don't echo back large blocks of code or file contents unless asked.
- Batch related edits into single operations. Don't make 5 edits when 1 handles it.
- Skip confirmations like "I'll continue..." Just do it.
- If a task needs 1 tool call, don't use 3. Plan before acting.
- Do not summarize what you just did unless the result is ambiguous or you need additional input.
