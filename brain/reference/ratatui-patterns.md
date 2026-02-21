# Ratatui Patterns Reference

## Event Handling Patterns

### Pattern 1: Blocking Event Loop (Simple)

```rust
use ratatui::crossterm::event::{self, Event, KeyCode};

fn run_app(&mut self) -> Result<()> {
    loop {
        // Draw
        terminal.draw(|f| self.ui(f))?;
        
        // Wait for input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('f') => self.feed(),
                _ => {}
            }
        }
    }
    Ok(())
}
```

**Use when**: Simple apps without real-time updates

### Pattern 2: Non-Blocking with Poll (Game Loop)

```rust
use ratatui::crossterm::event::{self, Event, KeyCode, poll};
use std::time::Duration;

fn run_app(&mut self) -> Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();
    
    loop {
        // Non-blocking check for input
        if poll(tick_rate.saturating_sub(last_tick.elapsed()))? {
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        
        // Game tick
        if last_tick.elapsed() >= tick_rate {
            self.on_tick();
            last_tick = Instant::now();
        }
        
        // Always draw
        terminal.draw(|f| self.ui(f))?;
    }
}
```

**Use when**: Real-time updates needed (stat decay, animations)

### Pattern 3: Async with Tokio

```rust
use tokio::time::{interval, Interval};

async fn run_app(&mut self) -> Result<()> {
    let mut tick = interval(Duration::from_millis(100));
    let mut reader = event::EventStream::new();
    
    loop {
        tokio::select! {
            // Timer tick
            _ = tick.tick() => {
                self.on_tick();
            }
            
            // Input event
            Some(Ok(event)) = reader.next() => {
                self.handle_event(event);
            }
        }
        
        terminal.draw(|f| self.ui(f))?;
    }
}
```

**Use when**: Complex async operations, multiple timers

## Widget Patterns

### Pattern 1: Implementing Custom Widget

```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
    style::{Color, Style},
    text::{Line, Span},
};

pub struct StatBar {
    label: String,
    value: u8,
    color: Color,
}

impl StatBar {
    pub fn new(label: &str, value: u8, color: Color) -> Self {
        Self {
            label: label.to_string(),
            value,
            color,
        }
    }
}

impl Widget for StatBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let filled = (area.width as u8 * self.value / 100) as usize;
        
        // Label
        buf.set_string(area.x, area.y, &self.label, Style::default());
        
        // Bar background
        let bar_area = Rect {
            x: area.x + 10,
            y: area.y,
            width: area.width - 10,
            height: 1,
        };
        
        // Filled portion
        for x in 0..filled {
            buf.set_string(
                bar_area.x + x as u16,
                bar_area.y,
                "█",
                Style::default().fg(self.color),
            );
        }
        
        // Empty portion
        for x in filled..bar_area.width as usize {
            buf.set_string(
                bar_area.x + x as u16,
                bar_area.y,
                "░",
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}
```

### Pattern 2: Stateful Widget

```rust
use ratatui::widgets::{StatefulWidget, List, ListState};

pub struct MenuWidget {
    items: Vec<String>,
}

pub struct MenuState {
    selected: usize,
}

impl StatefulWidget for MenuWidget {
    type State = MenuState;
    
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items: Vec<ListItem> = self.items
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let style = if i == state.selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                ListItem::new(text.clone()).style(style)
            })
            .collect();
        
        List::new(items).render(area, buf);
    }
}
```

### Pattern 3: Canvas for ASCII Art

```rust
use ratatui::widgets::canvas::{Canvas, Points};

fn draw_pet(frame: &mut Frame, area: Rect) {
    let canvas = Canvas::default()
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(|ctx| {
            // Draw ASCII art as points
            ctx.draw(&Points {
                coords: &[(10.0, 10.0), (11.0, 10.0), (10.0, 11.0)],
                color: Color::White,
            });
        });
    
    frame.render_widget(canvas, area);
}
```

## Layout Patterns

### Pattern 1: Constraint-Based Layout

```rust
use ratatui::layout::{Layout, Constraint, Direction};

fn calculate_layout(area: Rect) -> LayoutChunks {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(area);
    
    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Left (pet)
            Constraint::Percentage(50),  // Right (stats)
        ])
        .split(main[1]);
    
    LayoutChunks {
        header: main[0],
        pet_area: content[0],
        stats_area: content[1],
        footer: main[2],
    }
}
```

### Pattern 2: Centered Popup

```rust
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

// Usage
let popup_area = centered_rect(60, 20, frame.size());
frame.render_widget(Clear, popup_area);  // Clear background
frame.render_widget(popup_content, popup_area);
```

## Styling Patterns

### Pattern 1: Style Builder

```rust
use ratatui::style::{Color, Modifier, Style};

fn get_stat_style(value: u8) -> Style {
    let color = match value {
        0..=25 => Color::Red,
        26..=50 => Color::Yellow,
        _ => Color::Green,
    };
    
    Style::default()
        .fg(color)
        .add_modifier(Modifier::BOLD)
}
```

### Pattern 2: Theme System

```rust
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub background: Color,
    pub foreground: Color,
}

pub const DEFAULT_THEME: Theme = Theme {
    primary: Color::Cyan,
    secondary: Color::Magenta,
    success: Color::Green,
    warning: Color::Yellow,
    error: Color::Red,
    background: Color::Black,
    foreground: Color::White,
};

impl App {
    fn styled_text(&self, text: &str, style_type: StyleType) -> Span {
        let style = match style_type {
            StyleType::Success => Style::default().fg(self.theme.success),
            StyleType::Warning => Style::default().fg(self.theme.warning),
            StyleType::Error => Style::default().fg(self.theme.error),
        };
        Span::styled(text, style)
    }
}
```

## Input Handling Patterns

### Pattern 1: Command Pattern

```rust
#[derive(Debug, Clone, Copy)]
pub enum Command {
    Quit,
    Feed,
    Play,
    Clean,
    Sleep,
    Medicine,
    Select(usize),
    ScrollUp,
    ScrollDown,
}

impl App {
    fn handle_key(&mut self, key: KeyEvent
) -> Option<Command> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Command::Quit),
            KeyCode::Char('f') => Some(Command::Feed),
            KeyCode::Char('p') => Some(Command::Play),
            KeyCode::Char('c') => Some(Command::Clean),
            KeyCode::Char('s') => Some(Command::Sleep),
            KeyCode::Char('m') => Some(Command::Medicine),
            KeyCode::Up => Some(Command::ScrollUp),
            KeyCode::Down => Some(Command::ScrollDown),
            KeyCode::Char(c) if c.is_ascii_digit() => {
                Some(Command::Select(c.to_digit(10).unwrap() as usize))
            }
            _ => None,
        }
    }
}
```

### Pattern 2: Modal Input

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,      // Game commands
    Menu,        // Menu navigation
    TextInput,   // Typing text
}

impl App {
    fn handle_key(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Menu => self.handle_menu_key(key),
            InputMode::TextInput => self.handle_text_key(key),
        }
    }
    
    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('m') => self.input_mode = InputMode::Menu,
            KeyCode::Char('n') => self.input_mode = InputMode::TextInput,
            _ => self.handle_action_key(key),
        }
    }
}
```

## Error Handling in UI

### Pattern 1: Error Popup

```rust
pub struct App {
    error: Option<String>,
}

impl App {
    fn show_error(&mut self, msg: impl Into<String>) {
        self.error = Some(msg.into());
    }
    
    fn draw(&self, frame: &mut Frame) {
        // ... main UI ...
        
        if let Some(ref error) = self.error {
            let popup = centered_rect(70, 30, frame.size());
            let text = Paragraph::new(error.as_str())
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Red));
            
            frame.render_widget(Clear, popup);
            frame.render_widget(text, popup);
        }
    }
}
```

## Performance Tips

1. **Minimize allocations in render loop**: Pre-allocate buffers
2. **Use `render_stateful_widget` for lists**: Only renders visible items
3. **Cache expensive calculations**: Store computed layouts
4. **Batch draw operations**: Group similar operations
5. **Use `Clear` widget sparingly**: It's expensive, prefer incremental updates

## Testing Widgets

```rust
#[test]
fn test_stat_bar_rendering() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
    let bar = StatBar::new("Hunger", 50, Color::Red);
    
    bar.render(buf.area, &mut buf);
    
    // Check that half the bar is filled
    let filled_count = buf.content.iter()
        .filter(|c| c.symbol() == "█")
        .count();
    
    assert!(filled_count > 0 && filled_count < 20);
}
```
