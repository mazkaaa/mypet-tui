//! Terminal UI setup and event handling

use std::io;

use crate::app::App;
use crate::ui;

/// Terminal UI wrapper
pub struct Tui<'a> {
    terminal: &'a mut ratatui::DefaultTerminal,
}

impl<'a> Tui<'a> {
    /// Create a new TUI instance
    pub fn new(terminal: &'a mut ratatui::DefaultTerminal) -> Self {
        Self { terminal }
    }

    /// Draw the UI
    pub fn draw(&mut self, app: &App) -> io::Result<()> {
        self.terminal.draw(|frame| ui::render(frame, app))?;
        Ok(())
    }
}
