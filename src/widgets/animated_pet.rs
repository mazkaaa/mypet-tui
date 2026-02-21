use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::animation::engine::AnimationEngine;

#[derive(Debug)]
pub struct AnimatedPet {
    engine: AnimationEngine,
}

impl AnimatedPet {
    pub fn new() -> Self {
        let mut engine = AnimationEngine::new();
        engine.request(crate::animation::types::AnimationType::IdleNeutral);

        Self { engine }
    }

    pub fn trigger(&mut self, anim_type: crate::animation::types::AnimationType) {
        self.engine.request(anim_type);
    }

    pub fn update(&mut self) {
        self.engine.update();
    }

    pub fn set_idle(&mut self) {
        self.engine
            .request(crate::animation::types::AnimationType::IdleNeutral);
    }

    pub fn set_idle_happy(&mut self) {
        self.engine
            .request(crate::animation::types::AnimationType::IdleHappy);
    }

    pub fn set_idle_sad(&mut self) {
        self.engine
            .request(crate::animation::types::AnimationType::IdleSad);
    }

    pub fn set_idle_sleeping(&mut self) {
        self.engine
            .request(crate::animation::types::AnimationType::IdleSleeping);
    }
}

impl Default for AnimatedPet {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &AnimatedPet {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let fallback = "  (?.?)  ".to_string();
        let binding = [fallback];
        let art = self.engine.current_art().unwrap_or(&binding);

        let art_height = art.len() as u16;
        let art_width = art.first().map(|s| s.len() as u16).unwrap_or(0);

        let y_offset = area.height.saturating_sub(art_height) / 2;
        let x_offset = area.width.saturating_sub(art_width) / 2;

        let mut style = Style::default();
        if let Some(color) = self.engine.current_color() {
            style = style.fg(color);
        }

        for (i, line) in art.iter().enumerate() {
            let y = area.y + y_offset + i as u16;
            let x = area.x + x_offset;

            if y < area.y + area.height {
                buf.set_string(x, y, line, style);
            }
        }

        for particle in self.engine.particles() {
            let (px, py) = particle.position();
            let abs_x = area.x + x_offset + px;
            let abs_y = area.y + y_offset + py;

            if abs_x < area.x + area.width && abs_y < area.y + area.height {
                buf.set_string(
                    abs_x,
                    abs_y,
                    particle.spec.symbol.to_string(),
                    Style::default().fg(particle.spec.color),
                );
            }
        }
    }
}
