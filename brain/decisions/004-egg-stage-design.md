# ADR 004: Egg Stage Design

## Status

Accepted

## Context

The Egg stage is the player's first interaction with MyPet TUI. We needed to design an engaging onboarding experience that:
- Introduces players to the care mechanics
- Creates tension and emotional investment before the pet hatches
- Provides meaningful choices that affect future gameplay
- Teaches the pattern of monitoring stats and responding to needs

This ADR documents the design decisions made for the egg incubation mechanics.

## Decision

We implemented a time-based incubation system with an interactive warmth mechanic.

### Key Design Decisions

1. **Separate `EggStats` struct** instead of reusing `Stats`
2. **Time-based incubation** instead of warmth-based progression
3. **Health only visible when critical** (warmth < 30)
4. **Slower warmth decay** rate (-3 per 5 seconds)
5. **Game over mechanic** for egg neglect

## Rationale

### Decision 1: Separate EggStats Struct

**Option A: Reuse `Stats` struct**
- Map hunger → incubation progress, happiness → warmth, etc.
- Pros: Less code, consistent interface
- Cons: Confusing mapping, egg isn't "hungry", stat semantics don't match

**Option B: Create dedicated `EggStats` struct**
```rust
pub struct EggStats {
    pub incubation_progress: f32,  // 0-100%
    pub warmth_level: u8,          // 0-100
    pub health: u8,                // 0-100 (hidden until critical)
}
```
- Pros: Clear semantics, explicit purpose, easier to modify egg mechanics independently
- Cons: More code, another struct to maintain

**Decision**: Option B - Dedicated `EggStats` struct

**Rationale**: While reusing `Stats` would reduce code, it creates a confusing abstraction. An egg doesn't have "hunger" or "hygiene" - it has incubation progress and warmth. Using dedicated fields makes the code self-documenting and allows the egg mechanics to evolve independently from pet stats. The `Stats` system is designed for living creatures, not embryonic development.

---

### Decision 2: Time-Based Incubation

**Option A: Warmth-based progression**
- Egg only incubates when warmth is maintained
- Pros: Rewards active play, creates urgency
- Cons: Punishes players who look away, can stall indefinitely, frustrating if missed

**Option B: Time-based progression (chosen)**
- Egg incubates automatically over ~30 seconds regardless of warmth
- Warmth only affects health (and hatching bonus)
- Pros: Guaranteed progression, less frustrating, warmth is still important
- Cons: Less "realistic", players might ignore warmth until critical

**Decision**: Option B - Time-based incubation with warmth affecting health

**Rationale**: We wanted the egg stage to be engaging without being punishing. If incubation required constant warmth, a player who checks their phone for 30 seconds could stall progress indefinitely. Time-based incubation ensures the player sees progress and reaches the "reward" (hatching) in a reasonable timeframe. Warmth becomes a risk-management mechanic (don't let health drop) rather than a progression gate, which is more forgiving for new players learning the interface.

---

### Decision 3: Hidden Health Until Critical

**Option A: Always show health**
- Health bar visible at all times alongside warmth
- Pros: Complete information, no surprises
- Cons: Cluttered UI, health feels like a stat to "manage" constantly

**Option B: Only show health when warmth < 30 (chosen)**
- Health bar hidden when egg is comfortable (warmth ≥ 30)
- Health appears (with warning) when warmth drops too low
- Pros: Clean UI by default, creates dramatic tension, teaches cause-effect
- Cons: Players might not know health exists until it's a problem

**Decision**: Option B - Contextual health visibility

**Rationale**: This creates a more emotionally engaging experience. When the health bar suddenly appears with a warning message, it signals "something is wrong!" rather than being background noise. It also teaches players the relationship between warmth and health through discovery rather than explicit tutorial text. The UI remains clean and focused (just warmth and progress) until there's an actual problem to solve, at which point the UI adapts to show the relevant information.

---

### Decision 4: Warmth Decay Rate

**Option A: Fast decay** (-5 per 2 seconds)
- Requires pressing W every ~8 seconds to maintain
- Pros: High engagement, constant interaction
- Cons: Anxiety-inducing, punishing, feels like a clicker game

**Option B: Moderate decay** (-3 per 5 seconds)
- Requires pressing W every ~15 seconds to maintain
- Pros: Comfortable pace, allows brief pauses, less stressful
- Cons: Less "tense", might feel too easy

**Option C: Very slow decay** (-1 per 10 seconds)
- Requires pressing W every ~50 seconds
- Pros: Very relaxed, casual-friendly
- Cons: Barely a mechanic, players might forget about it entirely

**Decision**: Option B - Moderate decay (-3 per 5 seconds)

**Rationale**: We tested several rates and found -3 per 5 seconds strikes the right balance. It's frequent enough that players must actively engage with the egg (about 2-3 times per minute), but not so demanding that it becomes stressful. A player can read a message or grab a drink without returning to a dead egg. This rate also aligns with the 30-second incubation time - players will press W roughly 2-4 times during incubation, which feels like meaningful interaction without being tedious.

---

### Decision 5: Game Over Mechanic for Eggs

**Option A: No death - just delayed hatching**
- Egg can't die, but low health affects starting stats
- Pros: No "fail state", casual-friendly
- Cons: Removes tension, players can ignore warmth entirely

**Option B: Game over when health reaches 0 (chosen)**
- Egg dies if neglected too long, ending the game
- Pros: Creates genuine stakes, failure has consequence, memorable moment
- Cons: Frustrating if accidental, might discourage some players

**Decision**: Option B - Game over for egg neglect

**Rationale**: The original Tamagotchi games were memorable *because* pets could die. Death created emotional investment - players cared because loss was possible. Removing death would make the egg stage feel like a waiting period rather than a care mechanic. However, we balanced this with generous thresholds: a player would need to ignore the egg for ~20+ seconds while warmth is critically low before death occurs. This gives ample warning (visual health bar, warning messages, color changes) before the point of no return. The game over also serves as a tutorial for the main game - if you can't keep an egg alive for 30 seconds, you're not ready for the ongoing care of a pet.

## Implementation Details

### Warmth Thresholds and Messages

| Warmth | Health | Message |
|--------|--------|---------|
| ≥ 70 | N/A | "😊 The egg is cozy and warm!" |
| 50-69 | N/A | "😐 The egg feels comfortable." |
| 30-49 | N/A | "🙂 The egg is cool but okay." |
| 20-29 | Visible | "⚠️ The egg feels cold..." |
| < 20 | Visible | "🧊 The egg is freezing!" |
| < 30 | < 30 | "⚠️ CRITICAL: The egg is dying!" |

### Hatching Bonus Calculation

```rust
let health_ratio = final_health / 100.0;

if health_ratio > 0.9 {
    // +10 to all stats
    bonus.description = "Perfectly incubated! The baby is thriving!";
} else if health_ratio > 0.7 {
    // +5 happiness, +5 energy
    bonus.description = "Well cared for. The baby is healthy and happy.";
} else if health_ratio > 0.5 {
    // +3 happiness
    bonus.description = "Adequate care. The baby seems content.";
} else {
    // -5 hunger (penalty)
    bonus.description = "The egg was neglected. The baby seems weak.";
}
```

### Egg Death Timeline

With default settings, if warmth is allowed to reach 0:

1. **Warmth 30** → Health bar appears (warning)
2. **Warmth 20** → "Freezing" message, -2 health/second
3. **Warmth 0** → "Critical" message, -5 health/second
4. **~15-20 seconds** later → Health reaches 0 → Game Over

Total grace period from first warning to death: ~40-50 seconds, giving players ample time to respond.

## Consequences

### Positive

- **Emotional Investment**: Players care about the egg because loss is possible
- **Teaching Moment**: Egg stage teaches the pattern of "monitor stats → respond to needs" that continues in the main game
- **Meaningful Choices**: Good care = better starting stats, rewarding attentive players
- **Tension Without Frustration**: Danger is real but forgiving; attentive players will never see game over
- **Clean UI**: Health only appears when relevant, reducing cognitive load

### Negative

- **Potential Frustration**: New players might lose their first egg and be discouraged
- **Accessibility**: Players with motor impairments might struggle with the timing
- **Save Scumming**: JSON save format means players can reload if they fail

### Mitigations

- **Ample Warning**: Health bar + color changes + text warnings give multiple chances to recover
- **Fast Restart**: R key instantly restarts from egg stage (no long load)
- **No Progress Lost**: First hatching is only ~30 seconds, so failure isn't a huge setback
- **Visual Indicators**: Color-coded warmth bar (red/yellow/green) makes status obvious at a glance

## Future Considerations

- **Difficulty Settings**: Could add "Casual Mode" where eggs can't die
- **Tutorial Mode**: First-time players could get extra warnings or slower decay
- **Egg Variety**: Different egg types could have different incubation times or warmth requirements
- **Achievement**: "Perfect Hatch" for 100% health at hatching
- **Statistics**: Track best/worst egg care across playthroughs

## References

- [Original Tamagotchi death mechanics](https://tamagotchi.fandom.com/wiki/Death)
- [Game Feel: A Game Designer's Guide to Virtual Sensation](http://www.game-feel.com/) - Chapter on creating "juicy" feedback
- [ADR 003: Save File Format](./003-save-format.md) - JSON format allows save manipulation (intentional for single-player)
- Phase 2 Pet System documentation for complete egg mechanics

## Related Decisions

- **ADR 003**: Save format (JSON enables save reloads)
- **Phase 2**: Pet system implementation
- **State Management**: GameState enum handles Egg → Playing transitions
