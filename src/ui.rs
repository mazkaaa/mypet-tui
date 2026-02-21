//! UI rendering module

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, GameState};
use crate::pet::LifeStage;

/// Render the UI
pub fn render(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(frame.area());

    // Header
    let header = Block::default()
        .title(" MyPet TUI - v0.1.0 ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(header, main_layout[0]);

    // Main content area
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[1]);

    // Left side: Pet and Event Log
    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(content_layout[0]);

    // Pet display (top left)
    render_pet(frame, app, left_layout[0]);

    // Event log (bottom left)
    render_event_log(frame, app, left_layout[1]);

    // Stats panel (right side)
    render_stats(frame, app, content_layout[1]);

    // Actions bar at bottom
    render_actions(frame, app, main_layout[2]);
}

fn render_pet(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let pet_block = Block::default()
        .title(format!(
            " {} - {} ",
            app.pet.name,
            app.pet.stage.display_name()
        ))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(pet_block, area);

    let inner = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .margin(1)
        .split(area)[0];

    // Use animated pet for hatched stages, static art for egg
    if app.pet.stage == LifeStage::Egg {
        let art_color = Color::White;
        let pet_art = Paragraph::new(app.pet.stage.ascii_art())
            .alignment(Alignment::Center)
            .style(Style::default().fg(art_color));
        frame.render_widget(pet_art, inner);
    } else {
        frame.render_widget(&app.animated_pet, inner);
    }
}

fn render_stats(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let stats_block = Block::default()
        .title(" Stats ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(stats_block, area);

    // Check if we're in Egg stage
    if app.pet.stage == LifeStage::Egg {
        render_egg_stats(frame, app, area);
        return;
    }

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Hunger
            Constraint::Length(3), // Happiness
            Constraint::Length(3), // Energy
            Constraint::Length(3), // Health
            Constraint::Length(3), // Hygiene
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Age
            Constraint::Length(2), // Status
            Constraint::Min(0),    // Remaining space
        ])
        .margin(1)
        .split(area);

    // Render stat bars
    render_stat_bar(
        frame,
        "Hunger",
        app.pet.stats.hunger.value(),
        inner[1],
        Color::Red,
    );
    render_stat_bar(
        frame,
        "Happiness",
        app.pet.stats.happiness.value(),
        inner[2],
        Color::Green,
    );
    render_stat_bar(
        frame,
        "Energy",
        app.pet.stats.energy.value(),
        inner[3],
        Color::Blue,
    );
    render_stat_bar(
        frame,
        "Health",
        app.pet.stats.health.value(),
        inner[4],
        Color::Magenta,
    );
    render_stat_bar(
        frame,
        "Hygiene",
        app.pet.stats.hygiene.value(),
        inner[5],
        Color::Cyan,
    );

    // Age
    let age_text = format!("Age: {}", app.pet.age_formatted());
    let age = Paragraph::new(age_text).style(Style::default().fg(Color::White));
    frame.render_widget(age, inner[7]);

    // Status message
    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    frame.render_widget(status, inner[8]);
}

fn render_egg_stats(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let warmth = app.pet.get_warmth();
    let incubation = app.pet.get_incubation();
    let health = app.pet.get_egg_health();

    // Only show health if warmth is low
    let show_health = warmth < 30;

    let constraints = if show_health {
        vec![
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Incubation
            Constraint::Length(3), // Warmth
            Constraint::Length(3), // Health (critical)
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Age
            Constraint::Length(2), // Status
            Constraint::Min(0),    // Remaining space
        ]
    } else {
        vec![
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Incubation
            Constraint::Length(3), // Warmth
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Age
            Constraint::Length(2), // Status
            Constraint::Min(0),    // Remaining space
        ]
    };

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .margin(1)
        .split(area);

    // Incubation progress bar
    render_stat_bar(frame, "Incubation", incubation, inner[1], Color::Green);

    // Warmth bar (color changes based on level)
    let warmth_color = match warmth {
        0..=30 => Color::Red,
        31..=60 => Color::Yellow,
        _ => Color::Green,
    };
    render_stat_bar(frame, "Warmth", warmth, inner[2], warmth_color);

    // Health (only if warmth is low)
    if show_health {
        render_stat_bar(frame, "⚠ Health", health, inner[3], Color::Red);
    }

    // Age
    let age_idx = if show_health { 5 } else { 4 };
    let age_text = format!("Age: {}", app.pet.age_formatted());
    let age = Paragraph::new(age_text).style(Style::default().fg(Color::White));
    frame.render_widget(age, inner[age_idx]);

    // Status message
    let status_idx = if show_health { 6 } else { 5 };
    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    frame.render_widget(status, inner[status_idx]);
}

fn render_stat_bar(
    frame: &mut Frame,
    label: &str,
    value: u8,
    area: ratatui::layout::Rect,
    color: Color,
) {
    let gauge = Gauge::default()
        .block(Block::default().title(label).borders(Borders::NONE))
        .gauge_style(Style::default().fg(color).bg(Color::Black))
        .percent(value as u16)
        .label(format!("{}%", value));

    frame.render_widget(gauge, area);
}

fn render_event_log(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let event_block = Block::default()
        .title(" Event Log ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    frame.render_widget(event_block, area);

    let inner = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .margin(1)
        .split(area)[0];

    // Get recent events
    let recent_events = app.event_system.recent_events(5);

    let event_text = if recent_events.is_empty() {
        "No events yet...".to_string()
    } else {
        recent_events
            .iter()
            .map(|e| format!("> {}", e.message))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let events = Paragraph::new(event_text)
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });

    frame.render_widget(events, inner);
}

fn render_actions(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let actions_block = Block::default()
        .title(" Actions ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));

    frame.render_widget(actions_block, area);

    let inner = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .margin(1)
        .split(area)[0];

    // Check for game over state first
    let actions_text = if app.game_state == GameState::GameOver {
        Paragraph::new("[R]estart  [Q]uit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red))
    } else if app.pet.stage == LifeStage::Egg {
        Paragraph::new("[W]arm Egg  [Q]uit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
    } else if app.pet.stage == LifeStage::Baby {
        Paragraph::new("[F]eed  [P]lay (Gentle)  [C]lean  [S]leep  [M]edicine  [Q]uit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
    } else {
        Paragraph::new("[F]eed  [P]lay  [C]lean  [S]leep  [M]edicine  [Q]uit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
    };

    frame.render_widget(actions_text, inner);
}
