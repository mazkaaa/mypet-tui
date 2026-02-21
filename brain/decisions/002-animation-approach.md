# ADR 002: Animation Approach

## Status
Accepted

## Context

The pet needs visual life through animations. We need to decide on the animation architecture:
- Frame-based: Pre-drawn ASCII art frames played sequentially
- Procedural: Code-generated animations (moving parts mathematically)
- Hybrid: Combination of both

Key requirements:
- Idle animations (breathing, blinking) always playing
- State-based animations (happy, sad, sleeping)
- Action animations (eating, playing) triggered by user
- Smooth transitions between states
- Particle effects (hearts, sparkles)
- Reasonable memory usage
- Easy to add new animations

## Options Considered

### Option 1: Pure Frame-Based
**Description**: Every frame of animation is a complete ASCII art file

**Example**:
```
assets/animations/idle/frame_01.txt
assets/animations/idle/frame_02.txt
assets/animations/idle/frame_03.txt
```

**Pros**:
- Artist has full control over every frame
- Consistent visual style
- Simple to implement (just cycle through files)
- Easy to preview and tweak

**Cons**:
- High memory usage (each frame stored in RAM)
- Large asset size on disk
- Inflexible (can't modify at runtime)
- Many files to manage
- Smooth animations require many frames

### Option 2: Pure Procedural
**Description**: Animations generated mathematically by moving/changing ASCII characters

**Example**:
```rust
// Breathing: scale Y axis with sine wave
let scale = 1.0 + 0.1 * sin(time * 2.0);
render_pet_scaled(scale);
```

**Pros**:
- Very low memory usage
- Infinite variation
- Smooth by nature (no discrete frames)
- Small asset size

**Cons**:
- Complex to implement
- Limited to simple transformations
- Hard to create complex motions (eating, evolving)
- Requires mathematical modeling of every movement
- Difficult to achieve artistic vision

### Option 3: Hybrid (Frame + Procedural)
**Description**: Frame-based for complex animations, procedural for subtle effects

**Example**:
```rust
// Load frame-based animation
let frame = load_frame("eating", frame_number);

// Apply procedural breathing effect
let breath = sin(time) * 0.05;
render_with_transform(frame, scale_y: 1.0 + breath);
```

**Pros**:
- Best of both worlds
- Frames for complex actions (eating, evolving)
- Procedural for continuous effects (breathing, particles)
- Reasonable memory usage
- Flexible and extensible

**Cons**:
- More complex implementation
- Need to manage both systems
- Slightly higher CPU usage

### Option 4: Sprite-Based with Parts
**Description**: Break pet into parts (head, body, arms) that animate independently

**Example**:
```rust
struct PetSprite {
    head: SpritePart,
    body: SpritePart,
    left_arm: SpritePart,
    right_arm: SpritePart,
}
```

**Pros**:
- Efficient (reuse parts)
- Can combine animations
- Smoother transitions

**Cons**:
- Complex to implement
- Requires consistent art style
- Harder to create initial assets
- Overkill for ASCII art

## Decision

We will use **Option 3: Hybrid (Frame + Procedural)**.

## Rationale

1. **Idle Animations**: Procedural breathing/blinking gives life without storing many frames
2. **Action Animations**: Frame-based eating/playing allow detailed, controlled motion
3. **Particles**: Procedural system for hearts/sparkles that float and fade
4. **Memory Efficiency**: Only store necessary frames, generate continuous effects
5. **Artistic Control**: Frames for important moments, code for ambient effects

## Implementation Strategy

### Frame-Based Components
- Life stage ASCII art (egg, baby, child, teen, adult)
- Action sequences (eating, playing, cleaning)
- State transition animations (wake up, fall asleep, evolve)
- Emotion poses (happy jump, sad slump)

### Procedural Components
- Breathing effect (subtle scale on idle frames)
- Blinking (periodically replace eyes with closed version)
- Particle physics (hearts floating up, sparkles)
- Color pulsing (gentle brightness variation)
- Swaying (horizontal offset with sine wave)

## Consequences

### Positive
- Rich, varied animations possible
- Reasonable resource usage
- Easy to add new frame-based animations
- Smooth ambient effects

### Negative
- Two systems to maintain
- Need to synchronize procedural effects with frame changes
- More complex rendering pipeline

## Technical Notes

### Frame Storage Format
```rust
pub struct AnimationFrame {
    art: Vec<String>,           // ASCII art lines
    duration_ms: u64,
    particles: Vec<ParticleSpec>,
}
```

### Procedural Effects
```rust
pub struct ProceduralEffect {
    effect_type: EffectType,    // Breathing, Swaying, Pulse
    amplitude: f32,
    frequency: f32,
    phase: f32,
}
```

### Priority System
Frame-based animations take precedence:
1. Action animations (frame-based)
2. Transition animations (frame-based)
3. Emotion states (frame-based)
4. Idle base + procedural effects

## Open Questions

1. **Frame Rate**: Target 10 FPS for frame-based, 30 FPS for procedural
2. **Transition Smoothing**: Should we interpolate between frame-based animations?
3. **Asset Pipeline**: Create tool to preview animations from files?

## References

- WORKPLAN.md Animation System Design section
- ASCII art animation examples in gaming
- Ratatui canvas widget for rendering
