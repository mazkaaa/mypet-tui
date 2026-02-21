use ratatui::style::Color;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub art: Vec<String>,
    pub duration: Duration,
    pub color_override: Option<Color>,
    pub particles: Vec<ParticleSpec>,
}

impl AnimationFrame {
    pub fn new(art: Vec<String>) -> Self {
        Self {
            art,
            duration: Duration::from_millis(100),
            color_override: None,
            particles: vec![],
        }
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration = Duration::from_millis(ms);
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color_override = Some(color);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ParticleSpec {
    pub symbol: char,
    pub x_offset: i16,
    pub y_offset: i16,
    pub vx: f32,
    pub vy: f32,
    pub lifetime_ms: u64,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub spec: ParticleSpec,
    pub x: f32,
    pub y: f32,
    pub birth_time: std::time::Instant,
}

impl Particle {
    pub fn new(spec: ParticleSpec, base_x: u16, base_y: u16) -> Self {
        Self {
            x: base_x as f32 + spec.x_offset as f32,
            y: base_y as f32 + spec.y_offset as f32,
            spec,
            birth_time: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();
        self.x += self.spec.vx * dt_secs;
        self.y += self.spec.vy * dt_secs;
    }

    pub fn is_alive(&self) -> bool {
        self.birth_time.elapsed() < Duration::from_millis(self.spec.lifetime_ms)
    }

    pub fn position(&self) -> (u16, u16) {
        (self.x as u16, self.y as u16)
    }
}
