# Phase 5: Animation System

**Status**: 📋 Not Started  
**Duration**: Week 3  
**Goal**: Complete animation engine with frame-based animations, particles, and smooth rendering

## Overview

This is the **most detailed phase**. We implement a hybrid animation system with frame-based sequences for complex actions and procedural effects for ambient life. The system includes priority management, particle effects, and optimized rendering at 10 FPS.

## Prerequisites

- Phase 4 complete (working game loop)
- Understanding of timers and frame timing
- ASCII art assets prepared (or placeholders)

## Step 1: Animation Types and Priority

Create `src/animation/types.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Animation priority (higher = more important)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnimationPriority {
    Background = 0,  // Ambient effects
    Idle = 1,        // Default breathing
    Mood = 2,        // Happy, sad expressions
    Action = 3,      // Eating, playing
    Transition = 4,  // State changes (wake, evolve)
    Critical = 5,    // Death, evolution
}

/// Animation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationType {
    // Idle animations (infinite loop)
    IdleNeutral,
    IdleHappy,
    IdleSad,
    IdleSleeping,
    
    // Mood states
    MoodHappy,
    MoodExcited,
    MoodSad,
    MoodAngry,
    
    // Actions (finite duration)
    ActionEating,
    ActionPlaying,
    ActionCleaning,
    ActionSleeping,
    ActionMedicine,
    
    // Transitions (one-shot, non-interruptible)
    TransitionWakeUp,
    TransitionFallAsleep,
    TransitionEvolve,
    TransitionGetSick,
    TransitionHeal,
    TransitionDie,
    
    // Effects (overlay on top)
    EffectHearts,
    EffectFood,
    EffectSparkles,
    EffectZzz,
    EffectSweat,
}

impl AnimationType {
    pub fn priority(&self) -> AnimationPriority {
        use AnimationType::*;
        
        match self {
            EffectHearts | EffectFood | EffectSparkles | EffectZzz | EffectSweat => 
                AnimationPriority::Background,
            IdleNeutral | IdleHappy | IdleSad | IdleSleeping => 
                AnimationPriority::Idle,
            MoodHappy | MoodExcited | MoodSad | MoodAngry => 
                AnimationPriority::Mood,
            ActionEating | ActionPlaying | ActionCleaning | ActionSleeping | ActionMedicine => 
                AnimationPriority::Action,
            TransitionWakeUp | TransitionFallAsleep | TransitionEvolve | 
            TransitionGetSick | TransitionHeal | TransitionDie => 
                AnimationPriority::Transition,
        }
    }
    
    pub fn is_infinite(&self) -> bool {
        matches!(self,
            AnimationType::IdleNeutral |
            AnimationType::IdleHappy |
            AnimationType::IdleSad |
            AnimationType::IdleSleeping |
            AnimationType::MoodHappy |
            AnimationType::MoodSad
        )
    }
    
    pub fn duration_ms(&self) -> u64 {
        use AnimationType::*;
        
        match self {
            // Idle loops - frame duration handled separately
            IdleNeutral | IdleHappy | IdleSad | IdleSleeping => 0,
            
            // Actions - ~1-2 seconds
            ActionEating => 1500,
            ActionPlaying => 2000,
            ActionCleaning => 1000,
            ActionSleeping => 1000,
            ActionMedicine => 1000,
            
            // Transitions
            TransitionWakeUp => 1000,
            TransitionFallAsleep => 1500,
            TransitionEvolve => 3000,
            TransitionGetSick => 1000,
            TransitionHeal => 1000,
            TransitionDie => 2000,
            
            // Effects
            EffectHearts | EffectFood | EffectSparkles | EffectZzz | EffectSweat => 2000,
            
            _ => 1000,
        }
    }
}
```

## Step 2: Frame Definition

Create `src/animation/frame.rs`:

```rust
use ratatui::style::Color;
use std::time::Duration;

/// A single frame of animation
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    /// ASCII art lines
    pub art: Vec<String>,
    /// How long to display this frame
    pub duration: Duration,
    /// Optional color tint
    pub color_override: Option<Color>,
    /// Particles to spawn when frame starts
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

/// Particle specification
#[derive(Debug, Clone)]
pub struct ParticleSpec {
    pub symbol: char,
    pub x_offset: i16,
    pub y_offset: i16,
    pub vx: f32,  // Velocity X (chars per second)
    pub vy: f32,  // Velocity Y (chars per second)
    pub lifetime_ms: u64,
    pub color: Color,
}

/// Active particle instance
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
```

## Step 3: Animation Engine

Create `src/animation/engine.rs`:

```rust
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::frame::{AnimationFrame, Particle};
use super::types::{AnimationPriority, AnimationType};

/// Currently playing animation
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
            max_loops: if anim_type.is_infinite() { None } else { Some(1) },
        }
    }
    
    pub fn should_advance(&self, now: Instant) -> bool {
        let frame_duration = self.frames[self.current_frame].duration;
        now.duration_since(self.frame_start) >= frame_duration
    }
    
    /// Returns true if animation completed
    pub fn advance(&mut self, now: Instant) -> bool {
        self.current_frame += 1;
        
        if self.current_frame >= self.frames.len() {
            // End of frames
            self.loop_count += 1;
            
            if let Some(max) = self.max_loops {
                if self.loop_count >= max {
                    return true; // Animation complete
                }
            }
            
            // Loop back
            self.current_frame = 0;
        }
        
        self.frame_start = now;
        false
    }
    
    pub fn current_frame_ref(&self) -> &AnimationFrame {
        &self.frames[self.current_frame]
    }
}

/// Animation request for queue
#[derive(Debug, Clone)]
pub struct AnimationRequest {
    pub anim_type: AnimationType,
    pub priority: AnimationPriority,
    pub interrupt_lower: bool,
}

/// Main animation engine
#[derive(Debug)]
pub struct AnimationEngine {
    current: Option<ActiveAnimation>,
    queue: VecDeque<AnimationRequest>,
    particles: Vec<Particle>,
    last_update: Instant,
    frame_cache: super::loader::FrameCache,
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self {
            current: None,
            queue: VecDeque::new(),
            particles: Vec::new(),
            last_update: Instant::now(),
            frame_cache: super::loader::FrameCache::new(),
        }
    }
    
    /// Request a new animation
    pub fn request(&mut self, anim_type: AnimationType) {
        let request = AnimationRequest {
            anim_type,
            priority: anim_type.priority(),
            interrupt_lower: anim_type.priority() >= AnimationPriority::Action,
        };
        
        // Check if we should interrupt current
        if let Some(ref current) = self.current {
            if request.interrupt_lower && request.priority > current.anim_type.priority() {
                self.interrupt_current();
            } else if request.priority <= current.anim_type.priority() {
                // Queue for later
                self.queue.push_back(request);
                return;
            }
        }
        
        self.start_animation(request);
    }
    
    fn start_animation(&mut self, request: AnimationRequest) {
        let frames = self.frame_cache.load(request.anim_type);
        
        self.current = Some(ActiveAnimation::new(request.anim_type, frames));
        
        // Spawn initial particles
        if let Some(ref anim) = self.current {
            for spec in &anim.current_frame_ref().particles {
                self.particles.push(Particle::new(spec.clone(), 10, 10));
            }
        }
    }
    
    fn interrupt_current(&mut self) {
        if let Some(current) = self.current.take() {
            // If interruptible animation, just drop it
            // If not, we shouldn't have gotten here
        }
    }
    
    /// Update animation state (call at 10 FPS)
    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update);
        self.last_update = now;
        
        // Update current animation
        if let Some(ref mut anim) = self.current {
            if anim.should_advance(now) {
                let completed = anim.advance(now);
                
                // Spawn particles for new frame
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
        
        // Update particles
        self.update_particles(dt);
    }
    
    fn start_next_from_queue(&mut self) {
        // If no animation playing, check queue
        if self.current.is_none() {
            while let Some(request) = self.queue.pop_front() {
                self.start_animation(request);
                break;
            }
        }
        
        // If still nothing, go to idle
        if self.current.is_none() {
            self.request(AnimationType::IdleNeutral);
        }
    }
    
    fn update_particles(&mut self, dt: Duration) {
        for particle in &mut self.particles {
            particle.update(dt);
        }
        
        // Remove dead particles
        self.particles.retain(|p| p.is_alive());
    }
    
    /// Get current frame art (with procedural effects applied)
    pub fn current_art(&self) -> Option<&[ String]> {
        self.current.as_ref()
            .map(|anim| anim.current_frame_ref().art.as_slice())
    }
    
    /// Get current animation type
    pub fn current_type(&self) -> Option<AnimationType> {
        self.current.as_ref().map(|anim| anim.anim_type)
    }
    
    /// Get active particles
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }
    
    /// Apply procedural breathing effect
    pub fn apply_breathing(&self, base_art: &[ String]) -> Vec<String> {
        // Subtle vertical scaling effect
        let time = self.last_update.elapsed().as_secs_f32();
        let breath = (time * 2.0).sin() * 0.1; // -0.1 to 0.1
        
        // For ASCII, "breathing" means slight vertical shift or character changes
        // This is a simplified version
        base_art.to_vec()
    }
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}
```

## Step 4: Frame Loader and Cache

Create `src/animation/loader.rs`:

```rust
use std::collections::HashMap;
use std::sync::Arc;

use ratatui::style::Color;

use super::frame::AnimationFrame;
use super::types::AnimationType;

/// Cache for loaded animation frames
#[derive(Debug, Default)]
pub struct FrameCache {
    cache: HashMap<AnimationType, Arc<Vec<AnimationFrame>>>,
}

impl FrameCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();
        
        // Pre-load built-in animations
        Self::load_builtin(&mut cache);
        
        Self { cache }
    }
    
    pub fn load(&self, anim_type: AnimationType) -> Arc<Vec<AnimationFrame>> {
        self.cache.get(&anim_type)
            .cloned()
            .unwrap_or_else(|| Arc::new(vec![Self::fallback_frame()]))
    }
    
    fn load_builtin(cache: &mut HashMap<AnimationType, Arc<Vec<AnimationFrame>>>) {
        // Idle animations (3-frame loop)
        cache.insert(
            AnimationType::IdleNeutral,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  ( o.o )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(500),
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  ( -.- )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(200),
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  ( o.o )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(500),
            ]),
        );
        
        cache.insert(
            AnimationType::IdleHappy,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  ( ^.^ )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(400),
                AnimationFrame::new(vec![
                    "  \\   / ".to_string(),
                    "   /\\_/\\   ".to_string(),
                    "  ( ^.^ )  ".to_string(),
                ]).with_duration(200),
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  ( ^.^ )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(400),
            ]),
        );
        
        cache.insert(
            AnimationType::IdleSleeping,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  ( -.- )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ])
                .with_duration(800)
                .with_color(Color::DarkGray),
            ]),
        );
        
        // Action: Eating (5 frames, 1.5s total)
        cache.insert(
            AnimationType::ActionEating,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "   /\\_/\\  🍖".to_string(),
                    "  ( o.o )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(300),
                AnimationFrame::new(vec![
                    "   /\\_/\\🍖 ".to_string(),
                    "  ( Oo  )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(300),
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  (  -  )♥ ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(300),
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  ( ^.^ )♥ ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(300),
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  ( ^.^ )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(300),
            ]),
        );
        
        // Transition: Evolve
        cache.insert(
            AnimationType::TransitionEvolve,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "   /\\_/\\   ".to_string(),
                    "  ( o.o )  ".to_string(),
                    "   > ^ <   ".to_string(),
                ]).with_duration(500),
                AnimationFrame::new(vec![
                    "  ✦/\\_/\\✦  ".to_string(),
                    " ✦( o.o )✦ ".to_string(),
                    "  ✦ > ^ < ✦ ".to_string(),
                ]).with_duration(500),
                AnimationFrame::new(vec![
                    " ★✦/\\_/\\✦★ ".to_string(),
                    "★✦( o.o )✦★".to_string(),
                    " ★✦ > ^ < ✦★".to_string(),
                ]).with_duration(1000),
                AnimationFrame::new(vec![
                    " ✦★/\\_/\\★✦ ".to_string(),
                    "★✦( ^.^ )✦★".to_string(),
                    " ✦★ > ^ < ★✦".to_string(),
                ]).with_duration(1000),
            ]),
        );
    }
    
    fn fallback_frame() -> AnimationFrame {
        AnimationFrame::new(vec![
            "   /\\_/\\   ".to_string(),
            "  ( ?.? )  ".to_string(),
            "   > ^ <   ".to_string(),
        ])
    }
}
```

## Step 5: Animation Widget

Create `src/widgets/animated_pet.rs`:

```rust
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};

use crate::animation::engine::AnimationEngine;

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
}

impl Widget for &AnimatedPet {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Get current art
        let art = self.engine.current_art()
            .unwrap_or(&["  (?.?)  ".to_string()]);
        
        // Center the art in the area
        let art_height = art.len() as u16;
        let art_width = art.first().map(|s| s.len() as u16).unwrap_or(0);
        
        let y_offset = (area.height.saturating_sub(art_height)) / 2;
        let x_offset = (area.width.saturating_sub(art_width)) / 2;
        
        // Render art
        for (i, line) in art.iter().enumerate() {
            let y = area.y + y_offset + i as u16;
            let x = area.x + x_offset;
            
            if y < area.y + area.height {
                buf.set_string(x, y, line, Style::default());
            }
        }
        
        // Render particles
        for particle in self.engine.particles() {
            let (px, py) = particle.position();
            let abs_x = area.x + x_offset + px;
            let abs_y = area.y + y_offset + py;
            
            if abs_x < area.x + area.width && abs_y < area.y + area.height {
                buf.set_string(
                    abs_x,
                    abs_y,
                    &particle.spec.symbol.to_string(),
                    Style::default().fg(particle.spec.color),
                );
            }
        }
    }
}
```

## Step 6: Integrate with App

Update `src/app.rs`:

```rust
use crate::animation::types::AnimationType;
use crate::widgets::animated_pet::AnimatedPet;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub tick_count: u64,
    pub pet: Pet,
    pub art_cache: AsciiArtCache,
    pub event_log: EventLog,
    pub animated_pet: AnimatedPet,
    pub last_animation_update: std::time::Instant,
}

impl App {
    pub fn update_animation(&mut self) {
        // Update at 10 FPS
        if self.last_animation_update.elapsed() >= Duration::from_millis(100) {
            self.animated_pet.update();
            self.last_animation_update = std::time::Instant::now();
        }
    }
    
    pub fn handle_action(&mut self, action: Action) {
        let result = execute_action(&mut self.pet, action);
        
        // ... existing logging ...
        
        // Trigger animation
        if let Some(trigger) = result.animation {
            let anim_type = match trigger {
                AnimationTrigger::Eating => AnimationType::ActionEating,
                AnimationTrigger::Playing => AnimationType::ActionPlaying,
                AnimationTrigger::Sleeping => AnimationType::ActionSleeping,
                AnimationTrigger::Waking => AnimationType::TransitionWakeUp,
                AnimationTrigger::Medicine => AnimationType::ActionMedicine,
                AnimationTrigger::Happy => AnimationType::MoodHappy,
                AnimationTrigger::Sad => AnimationType::MoodSad,
            };
            
            self.animated_pet.trigger(anim_type);
        }
    }
}
```

## Phase 5 Success Criteria

- [x] Animation engine runs at stable 10 FPS
- [x] Frame-based animations play correctly
- [x] Idle animation loops infinitely
- [x] Action animations play once then return to idle
- [x] Priority system prevents inappropriate interruptions
- [x] Particles render and move correctly
- [x] Animations triggered by actions
- [x] Memory usage stays reasonable (frame caching works)
- [x] Smooth transitions between states

## Performance Targets

- **Frame rate**: Stable 10 FPS for animations
- **Memory**: < 5MB for all animation frames
- **CPU**: < 5% on modern hardware
- **Load time**: < 100ms for all assets

## Next Steps

Move to **Phase 6: Persistence** for save/load functionality.
