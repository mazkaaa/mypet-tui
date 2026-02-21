# ADR 001: Ratatui Framework Selection

## Status
Accepted

## Context

We need to select a TUI (Terminal User Interface) framework for building MyPet. The framework must:
- Support Rust (our primary language)
- Handle terminal initialization, raw mode, and cleanup
- Provide widget system for UI components
- Support event handling (keyboard input)
- Be actively maintained with good documentation
- Support async/concurrent updates (for game loop)

## Options Considered

### Option 1: Ratatui
**Description**: Modern Rust TUI library, fork of tui-rs with active development

**Pros**:
- Very active development (frequent releases)
- Excellent documentation at ratatui.rs
- Large community, many examples
- Built on crossterm (cross-platform)
- Async-friendly design
- Rich widget ecosystem
- Good performance
- Immediate mode rendering (simple mental model)

**Cons**:
- Relatively new (forked 2022), API still evolving
- Steep learning curve for complex layouts
- Requires manual state management

### Option 2: tui-rs (original)
**Description**: The original library that Ratatui forked from

**Pros**:
- Stable, well-established API
- Many existing projects use it
- Good documentation

**Cons**:
- No longer actively maintained (as of 2023)
- Security updates and bug fixes uncertain
- Missing newer features

### Option 3: Cursive
**Description**: TUI library with retained mode (widget tree) approach

**Pros**:
- Higher-level abstractions
- Built-in menu/dialog systems
- Easier for simple UIs

**Cons**:
- Less flexible for custom rendering
- Heavier dependency
- Smaller community than Ratatui
- Not as async-friendly

### Option 4: Termion + Custom
**Description**: Low-level terminal control, build UI from scratch

**Pros**:
- Full control over rendering
- Minimal dependencies
- Good learning experience

**Cons**:
- Significantly more code to write
- Must handle all terminal quirks
- Reinventing the wheel

## Decision

We will use **Ratatui** (v0.30.0 or later).

## Rationale

1. **Active Maintenance**: Ratatui is actively maintained with regular releases and responsive maintainers
2. **Documentation**: Comprehensive docs, book, and examples at ratatui.rs
3. **Community**: Growing Discord community, many real-world projects
4. **Architecture**: Immediate mode rendering matches our game loop model
5. **Async Support**: Works well with tokio for our timer-based game loop
6. **Future-Proof**: Positioned as the successor to tui-rs

## Consequences

### Positive
- Access to latest features and bug fixes
- Rich example ecosystem to learn from
- Good performance out of the box
- Cross-platform terminal support via crossterm

### Negative
- API may have breaking changes in minor versions
- Need to follow migration guides when upgrading
- Some advanced features require deeper understanding

## Related Decisions

- **Backend**: Use crossterm (Ratatui's default)
- **Async Runtime**: Use tokio (works well with Ratatui's event loop)

## References

- [Ratatui Documentation](https://ratatui.rs/)
- [Ratatui GitHub](https://github.com/ratatui/ratatui)
- [Comparison of Rust TUI libraries](https://blog.logrocket.com/rust-tui-libraries/)
