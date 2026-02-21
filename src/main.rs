//! MyPet TUI - A terminal-based virtual pet game

use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

mod animation;
mod app;
mod events;
mod pet;
mod stats;
mod tui;
mod ui;
mod widgets;

use app::App;
use tui::Tui;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();

    let result = run_app(&mut terminal, &mut app).await;

    ratatui::restore();
    result
}

async fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> io::Result<()> {
    let mut tui = Tui::new(terminal);
    let mut last_tick = tokio::time::Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        // Update app state
        app.tick();

        // Draw UI
        tui.draw(app)?;

        // Handle timeout for tick updates
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Poll for events with timeout
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Char('r') => app.restart(),
                        KeyCode::Char('w') => app.warm_egg(),
                        KeyCode::Char('f') => app.feed_pet(),
                        KeyCode::Char('p') => app.play_with_pet(),
                        KeyCode::Char('c') => app.clean_pet(),
                        KeyCode::Char('s') => app.toggle_sleep(),
                        KeyCode::Char('m') => app.give_medicine(),
                        _ => {}
                    }
                }
            }
        }

        // Update tick timer
        if last_tick.elapsed() >= tick_rate {
            last_tick = tokio::time::Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
