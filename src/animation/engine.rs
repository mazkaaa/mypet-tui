use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::frame::{AnimationFrame, Particle};
use super::loader::FrameCache;
use super::types::{AnimationPriority, AnimationType};

#[derive(Debug, Clone)]
pub struct ActiveAnimation {
    pub anim_type: AnimationType,
    pub frames: Arc<Vec<AnimationFrame>>,
    pub current_frame: usize,
    pub frame_start: Instant,
    pub loop_count: u32,
    pub max_loops: Option<u32>,
}

impl ActiveAnimation {
    pub fn new(anim_type: AnimationType, frames: Arc<Vec<AnimationFrame>>) -> Self {
        Self {
            anim_type,
            frames,
            current_frame: 0,
            frame_start: Instant::now(),
            loop_count: 0,
            max_loops: if anim_type.is_infinite() {
                None
            } else {
                Some(1)
            },
        }
    }

    pub fn should_advance(&self, now: Instant) -> bool {
        let frame_duration = self.frames[self.current_frame].duration;
        now.duration_since(self.frame_start) >= frame_duration
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        self.current_frame += 1;

        if self.current_frame >= self.frames.len() {
            self.loop_count += 1;

            if let Some(max) = self.max_loops
                && self.loop_count >= max {
                // Reset to valid frame before returning completion
                self.current_frame = self.frames.len().saturating_sub(1);
                self.frame_start = now;
                return true;
            }

            self.current_frame = 0;
        }

        self.frame_start = now;
        false
    }

    pub fn current_frame_ref(&self) -> &AnimationFrame {
        &self.frames[self.current_frame]
    }
}

#[derive(Debug, Clone)]
pub struct AnimationRequest {
    pub anim_type: AnimationType,
    pub priority: AnimationPriority,
    pub interrupt_lower: bool,
}

pub struct AnimationEngine {
    current: Option<ActiveAnimation>,
    queue: VecDeque<AnimationRequest>,
    particles: Vec<Particle>,
    last_update: Instant,
    frame_cache: FrameCache,
}

impl std::fmt::Debug for AnimationEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationEngine")
            .field("current", &self.current.is_some())
            .field("queue_len", &self.queue.len())
            .field("particles_len", &self.particles.len())
            .finish()
    }
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self {
            current: None,
            queue: VecDeque::new(),
            particles: Vec::new(),
            last_update: Instant::now(),
            frame_cache: FrameCache::new(),
        }
    }

    pub fn request(&mut self, anim_type: AnimationType) {
        let request = AnimationRequest {
            anim_type,
            priority: anim_type.priority(),
            interrupt_lower: anim_type.priority() >= AnimationPriority::Action,
        };

        if let Some(ref current) = self.current {
            if request.interrupt_lower && request.priority > current.anim_type.priority() {
                self.interrupt_current();
            } else if request.priority <= current.anim_type.priority() {
                self.queue.push_back(request);
                return;
            }
        }

        self.start_animation(request);
    }

    fn start_animation(&mut self, request: AnimationRequest) {
        let frames = self.frame_cache.load(request.anim_type);

        self.current = Some(ActiveAnimation::new(request.anim_type, frames));

        if let Some(ref anim) = self.current {
            for spec in &anim.current_frame_ref().particles {
                self.particles.push(Particle::new(spec.clone(), 10, 10));
            }
        }
    }

    fn interrupt_current(&mut self) {
        if let Some(_current) = self.current.take() {}
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update);
        self.last_update = now;

        if let Some(ref mut anim) = self.current {
            if anim.should_advance(now) {
                let completed = anim.advance(now);

                if let Some(ref anim) = self.current {
                    for spec in &anim.current_frame_ref().particles {
                        self.particles.push(Particle::new(spec.clone(), 10, 10));
                    }
                }

                if completed {
                    self.current = None;
                    self.start_next_from_queue();
                }
            }
        } else {
            self.start_next_from_queue();
        }

        self.update_particles(dt);
    }

    fn start_next_from_queue(&mut self) {
        if self.current.is_none() {
            if let Some(request) = self.queue.pop_front() {
                self.start_animation(request);
            }
        }

        if self.current.is_none() {
            self.request(AnimationType::IdleNeutral);
        }
    }

    fn update_particles(&mut self, dt: Duration) {
        for particle in &mut self.particles {
            particle.update(dt);
        }

        self.particles.retain(|p| p.is_alive());
    }

    pub fn current_art(&self) -> Option<&[String]> {
        self.current
            .as_ref()
            .map(|anim| anim.current_frame_ref().art.as_slice())
    }

    #[allow(dead_code)]
    pub fn current_type(&self) -> Option<AnimationType> {
        self.current.as_ref().map(|anim| anim.anim_type)
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn current_color(&self) -> Option<ratatui::style::Color> {
        self.current
            .as_ref()
            .and_then(|anim| anim.current_frame_ref().color_override)
    }
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}
