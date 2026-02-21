# Animation System Architecture

## Overview

This document details the complete animation system architecture, from frame definition to screen rendering.

## Design Goals

1. **Smooth**: Consistent frame rate (10 FPS target)
2. **Responsive**: Immediate response to state changes
3. **Efficient**: Minimal CPU/memory overhead
4. **Extensible**: Easy to add new animations
5. **Interruptible**: Higher priority animations can override lower ones

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Animation System                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │   Priority   │  │   Engine     │  │   Asset Manager      │  │
│  │   Queue      │──│   Core       │──│   (Lazy Loading)     │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│         │                 │                    │               │
│         ▼                 ▼                    ▼               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │   Request    │  │   Active     │  │   Frame Cache        │  │
│  │   Handler    │  │   Animation  │  │   (Pre-loaded)       │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                           │                                    │
│                           ▼                                    │
│                    ┌──────────────┐                            │
│                    │   Renderer   │──▶ Ratatui Canvas          │
│                    │   (Widget)   │                            │
│                    └──────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## Core Types

### Animation Priority System

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnimationPriority {
    Background = 0,    // Ambient effects (rain, day/night)
    Idle = 1,          // Default breathing, blinking
    Mood = 2,          // Happy, sad expressions
    Action = 3,        // Eating, playing (interrupts idle)
    Transition = 4,    // State changes (must complete)
    Critical = 5,      // Death, evolution (cannot be interrupted)
}

pub struct AnimationRequest {
    pub priority: AnimationPriority,
    pub animation_type: AnimationType,
    pub interrupt_lower: bool,  // Cancel lower priority animations
}
```

### Animation Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationType {
    // Idle animations (loop forever)
    IdleNeutral,
    IdleHappy,
    IdleSad,
    IdleSleeping,
    
    // Mood states
    MoodHappy,
    MoodExcited,
    MoodSad,
    MoodAngry,
    
    // Actions (triggered by user)
    ActionEating,
    ActionPlaying,
    ActionCleaning,
    ActionSleeping,
    ActionMedicine,
    
    // Transitions (play once)
    TransitionWakeUp,
    TransitionFallAsleep,
    TransitionEvolve,
    TransitionGetSick,
    TransitionHeal,
    
    // Effects (overlay)
    EffectHearts,
    EffectFood,
    EffectSparkles,
    EffectZzz,
}
```

### Active Animation State

```rust
pub struct ActiveAnimation {
    pub animation_type: AnimationType,
    pub frames: Arc<Vec<AnimationFrame>>,
    pub current_frame: usize,
    pub frame_start: Instant,
    pub loop_count: u32,
    pub max_loops: Option<u32>,  // None = infinite
    pub priority: AnimationPriority,
}

impl ActiveAnimation {
    /// Check if it's time to advance to next frame
    pub fn should_advance(&self, now: Instant) -> bool {
        let frame_duration = self.frames[self.current_frame].duration;
        now.duration_since(self.frame_start) >= frame_duration
    }
    
    /// Advance to next frame, return true if animation complete
    pub fn advance(&mut self, now: Instant) -> bool {
        self.current_frame += 1;
        
        if self.current_frame >= self.frames.len() {
            // End of frame sequence
            self.loop_count += 1;
            
            if let Some(max) = self.max_loops {
                if self.loop_count >= max {
                    return true; // Animation complete
                }
            }
            
            // Loop back to start
            self.current_frame = 0;
        }
        
        self.frame_start = now;
        false
    }
}
```

### Frame Definition

```rust
pub struct AnimationFrame {
    /// Multi-line ASCII art for this frame
    pub art: Vec<String>,
    
    /// Frame display duration
    pub duration: Duration,
    
    /// Optional color override (applies tint to entire frame)
    pub color_override: Option<Color>,
    
    /// Particles to spawn at this frame
    pub particles: Vec<ParticleSpec>,
    
    /// Sound effect trigger (future)
    pub sound: Option<SoundId>,
}

pub struct ParticleSpec {
    pub symbol: char,
    pub spawn_position: (i16, i16),  // Offset from center
    pub velocity: (f32, f32),        // Movement per second
    pub lifetime: Duration,
    pub color: Color,
}
```

## Animation Engine

```rust
pub struct AnimationEngine {
    /// Currently playing animation
    current: Option<ActiveAnimation>,
    
    /// Queue of pending animations (by priority)
    queue: BinaryHeap<AnimationRequest>,
    
    /// Active particle effects
    particles: Vec<Particle>,
    
    /// Asset loader/cache
    assets: AssetManager,
    
    /// Last update timestamp
    last_update: Instant,
}

impl AnimationEngine {
    pub fn request(&mut self, request: AnimationRequest) {
        // Check if we should interrupt current animation
        if let Some(ref current) = self.current {
            if request.interrupt_lower && request.priority > current.priority {
                // Move current to queue if it can be resumed
                self.interrupt_current();
            } else if request.priority <= current.priority {
                // Queue for later
                self.queue.push(request);
                return;
            }
        }
        
        // Start new animation
        self.start_animation(request);
    }
    
    fn start_animation(&mut self, request: AnimationRequest) {
        let frames = self.assets.load_animation(request.animation_type);
        
        self.current = Some(ActiveAnimation {
            animation_type: request.animation_type,
            frames,
            current_frame: 0,
            frame_start: Instant::now(),
            loop_count: 0,
            max_loops: Self::get_loop_count(request.animation_type),
            priority: request.priority,
        });
    }
    
    pub fn update(&mut self, now: Instant) {
        let dt = now.duration_since(self.last_update);
        self.last_update = now;
        
        // Update current animation
        if let Some(ref mut anim) = self.current {
            if anim.should_advance(now) {
                let complete = anim.advance(now);
                
                // Spawn particles for new frame
                self.spawn_particles(&anim.frames[anim.current_frame]);
                
                if complete {
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
    
    fn update_particles(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();
        
        self.particles.retain_mut(|p| {
            p.position.0 += p.velocity.0 * dt_secs;
            p.position.1 += p.velocity.1 * dt_secs;
            p.lifetime = p.lifetime.saturating_sub(dt);
            p.lifetime > Duration::ZERO
        });
    }
}
```

## Asset Management

### Lazy Loading with Caching

```rust
pub struct AssetManager {
    cache: HashMap<AnimationType, Arc<Vec<AnimationFrame>>>,
    load_queue: VecDeque<AnimationType>,
}

impl AssetManager {
    pub fn load_animation(&mut self, anim_type: AnimationType) -> Arc<Vec<AnimationFrame>> {
        // Return cached if available
        if let Some(cached) = self.cache.get(&anim_type) {
            return Arc::clone(cached);
        }
        
        // Load from disk
        let frames = self.load_from_disk(anim_type);
        let arc = Arc::new(frames);
        self.cache.insert(anim_type, Arc::clone(&arc));
        arc
    }
    
    fn load_from_disk(&self, anim_type: AnimationType) -> Vec<AnimationFrame> {
        let path = format!("assets/animations/{}", anim_type.asset_path());
        
        // Load frame files
        let mut frames = Vec::new();
        let frame_count = self.count_frames(&path);
        
        for i in 0..frame_count {
            let frame_path = format!("{}/frame_{:02}.txt", path, i);
            let art = fs::read_to_string(&frame_path)
                .unwrap_or_default()
                .lines()
                .map(|s| s.to_string())
                .collect();
            
            frames.push(AnimationFrame {
                art,
                duration: Duration::from_millis(100),
                color_override: None,
                particles: vec![],
                sound: None,
            });
        }
        
        frames
    }
    
    /// Pre-load common animations at startup
    pub fn preload_common(&mut self) {
        let common = [
            AnimationType::IdleNeutral,
            AnimationType::IdleHappy,
            AnimationType::ActionEating,
        ];
        
        for anim in &common {
            self.load_animation(*anim);
        }
    }
}
```

## Frame-Based vs Procedural

We use a **hybrid approach**:

### Frame-Based (Primary)
- Complex animations (eating, evolving)
- State transitions
- Life stage-specific art

**Pros**: Artist-controlled, consistent
**Cons**: Memory usage, fixed variations

### Procedural (Secondary)
- Idle breathing (subtle scale)
- Particle movement
- Color pulsing effects

**Pros**: Low memory, infinite variation
**Cons**: Limited complexity

```rust
impl AnimationEngine {
    /// Apply procedural effects to frame-based animation
    pub fn apply_procedural_effects(&self, frame: &mut RenderedFrame, base: &AnimationFrame) {
        // Breathing effect (subtle vertical stretch)
        if matches!(self.current.animation_type, AnimationType::IdleNeutral) {
            let breath = (self.last_update.elapsed().as_secs_f32() * 2.0).sin();
            frame.scale_y = 1.0 + (breath * 0.05);
        }
        
        // Color pulsing
        if let Some(base_color) = base.color_override {
            let pulse = (self.last_update.elapsed().as_secs_f32() * 3.0).sin();
            let intensity = ((pulse + 1.0) / 2.0 * 0.3 + 0.7) as u8;
            frame.color = base_color.scale(intensity);
        }
    }
}
```

## Memory Optimization

### Frame Data Structures

```rust
// Compact frame storage
pub struct CompactFrame {
    /// RLE-encoded art (for large frames)
    art_data: Vec<u8>,
    width: u8,
    height: u8,
    duration_ms: u16,
    color_idx: u8,  // Index into palette
}

/// Shared frame data across animations
pub struct FramePool {
    frames: Vec<CompactFrame>,
    animations: HashMap<AnimationType, Vec<usize>>, // Indices into frames
}
```

### Memory Budget

- **Target**: < 10MB for all animations
- **Idle animations**: 4 frames × 20 lines × 40 chars = ~3KB each
- **Action animations**: 8 frames average = ~6KB each
- **Cache limit**: LRU cache with 50 animation max

## State Priority Rules

```rust
impl AnimationEngine {
    /// Determine default animation from pet state
    pub fn select_idle_animation(&self, pet: &Pet) -> AnimationType {
        match pet.state {
            PetState::Sleeping { .. } => AnimationType::IdleSleeping,
            _ => {
                // Select based on happiness
                if pet.stats.happiness.value() > 70 {
                    AnimationType::IdleHappy
                } else if pet.stats.happiness.value() < 30 {
                    AnimationType::IdleSad
                } else {
                    AnimationType::IdleNeutral
                }
            }
        }
    }
    
    /// Handle state change - trigger appropriate transition
    pub fn on_state_change(&mut self, from: PetState, to: PetState) {
        let transition = match (from, to) {
            (PetState::Normal, PetState::Sleeping { .. }) => 
                Some(AnimationType::TransitionFallAsleep),
            (PetState::Sleeping { .. }, PetState::Normal) => 
                Some(AnimationType::TransitionWakeUp),
            (PetState::Normal, PetState::Sick { .. }) => 
                Some(AnimationType::TransitionGetSick),
            _ => None,
        };
        
        if let Some(anim) = transition {
            self.request(AnimationRequest {
                priority: AnimationPriority::Transition,
                animation_type: anim,
                interrupt_lower: true,
            });
        }
    }
}
```

## Performance Considerations

### 1. Frame Rate Independence

```rust
// Don't hard-code frame durations
const TARGET_FPS: f32 = 10.0;
const FRAME_DURATION: Duration = Duration::from_millis((1000.0 / TARGET_FPS) as u64);
```

### 2. Skip Frames if Behind

```rust
pub fn update(&mut self, now: Instant) {
    let elapsed = now.duration_since(self.last_update);
    
    // If we're more than 2 frames behind, skip to current
    if elapsed > FRAME_DURATION * 2 {
        self.catch_up(elapsed);
    }
}
```

### 3. Render Only Changed Regions

```rust
// Mark animation widget as dirty only when frame changes
if self.current_frame_changed {
    ctx.mark_dirty(self.widget_rect);
}
```

## Testing Animations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_animation_priority() {
        let mut engine = AnimationEngine::new();
        
        // Start idle
        engine.request(AnimationRequest {
            priority: AnimationPriority::Idle,
            animation_type: AnimationType::IdleNeutral,
            interrupt_lower: false,
        });
        
        // Action should interrupt
        engine.request(AnimationRequest {
            priority: AnimationPriority::Action,
            animation_type: AnimationType::ActionEating,
            interrupt_lower: true,
        });
        
        assert_eq!(
            engine.current.as_ref().unwrap().animation_type,
            AnimationType::ActionEating
        );
    }
    
    #[test]
    fn test_particle_lifetime() {
        let mut engine = AnimationEngine::new();
        
        engine.particles.push(Particle {
            lifetime: Duration::from_millis(100),
            ..Default::default()
        });
        
        engine.update_particles(Duration::from_millis(50));
        assert_eq!(engine.particles.len(), 1);
        
        engine.update_particles(Duration::from_millis(100));
        assert_eq!(engine.particles.len(), 0);
    }
}
```
