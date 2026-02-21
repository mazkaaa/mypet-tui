//! Event system for random occurrences and special moments

use std::time::{Duration, Instant};

use crate::pet::{LifeStage, Pet, PetState};

/// Generate a random float between 0.0 and 1.0
fn random_float() -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut hasher = DefaultHasher::new();
    nanos.hash(&mut hasher);
    let hash = hasher.finish();

    (hash as f64 / u64::MAX as f64) as f32
}

/// Types of events that can occur
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    /// Pet made a mess (hygiene drop)
    MadeMess,
    /// Pet found something interesting
    FoundTreasure,
    /// Pet had a bad dream (baby only)
    BadDream,
    /// Pet learned something new
    LearnedTrick,
    /// Evolution/milestone reached
    Evolved,
    /// Pet is very happy (special moment)
    HappyMoment,
    /// Pet is lonely (happiness low)
    Lonely,
    /// Pet is hungry and asking for food
    AskingForFood,
}

impl EventType {
    /// Get the message for this event
    pub fn message(&self, pet_name: &str) -> String {
        match self {
            EventType::MadeMess => format!("{} made a mess! (-20 hygiene)", pet_name),
            EventType::FoundTreasure => {
                format!("{} found a shiny object! (+10 happiness)", pet_name)
            }
            EventType::BadDream => format!("{} had a bad dream... (-10 happiness)", pet_name),
            EventType::LearnedTrick => format!("{} learned a new trick! (+15 happiness)", pet_name),
            EventType::Evolved => format!("{} evolved! 🎉", pet_name),
            EventType::HappyMoment => {
                format!("{} is having a wonderful time! (+20 happiness)", pet_name)
            }
            EventType::Lonely => format!("{} seems lonely...", pet_name),
            EventType::AskingForFood => {
                format!("{} is looking at you with hungry eyes...", pet_name)
            }
        }
    }
}

/// A game event with timestamp
#[derive(Debug, Clone)]
pub struct GameEvent {
    pub event_type: EventType,
    pub timestamp: Instant,
    pub message: String,
}

/// Event system that manages random occurrences
#[derive(Debug)]
pub struct EventSystem {
    /// Last time an event was triggered
    last_event_time: Instant,
    /// Minimum time between events
    event_cooldown: Duration,
    /// History of recent events
    pub event_history: Vec<GameEvent>,
    /// Maximum events to keep in history
    max_history: usize,
    /// Pending event to display
    pub pending_event: Option<GameEvent>,
}

impl EventSystem {
    /// Create a new event system
    pub fn new() -> Self {
        Self {
            last_event_time: Instant::now(),
            event_cooldown: Duration::from_secs(15), // Events every 15 seconds max
            event_history: Vec::new(),
            max_history: 10,
            pending_event: None,
        }
    }

    /// Update and potentially trigger events
    pub fn update(&mut self, pet: &mut Pet, _delta_time: Duration) {
        // Only trigger events if enough time has passed
        if self.last_event_time.elapsed() < self.event_cooldown {
            return;
        }

        // Don't trigger events for dead pets
        if !pet.state.is_alive() {
            return;
        }

        // Don't trigger events during sleep
        if pet.state.is_sleeping() {
            return;
        }

        // Check for random events (5% chance per update after cooldown)
        if random_float() < 0.05 {
            self.try_trigger_event(pet);
        }
    }

    /// Try to trigger a random event based on pet state
    fn try_trigger_event(&mut self, pet: &mut Pet) {
        let event_type = self.select_event_type(pet);

        if let Some(event_type) = event_type {
            // Apply event effects
            self.apply_event_effects(event_type.clone(), pet);

            // Create event
            let event = GameEvent {
                event_type: event_type.clone(),
                timestamp: Instant::now(),
                message: event_type.message(&pet.name),
            };

            // Add to history
            self.event_history.push(event.clone());
            if self.event_history.len() > self.max_history {
                self.event_history.remove(0);
            }

            // Set as pending for display
            self.pending_event = Some(event);

            // Reset cooldown
            self.last_event_time = Instant::now();
        }
    }

    /// Select an appropriate event type based on pet state
    fn select_event_type(&self, pet: &Pet) -> Option<EventType> {
        use EventType::*;

        let mut possible_events = Vec::new();

        // Check for evolution
        // This is handled separately in the pet update, but we can add a celebration event

        // Mess event (more likely when hygiene is high - making messes!)
        if pet.stats.hygiene.value() > 50 && pet.stage != LifeStage::Egg {
            possible_events.push((MadeMess, 0.2));
        }

        // Found treasure (random happy event)
        if pet.stats.happiness.value() > 40 {
            possible_events.push((FoundTreasure, 0.15));
        }

        // Bad dream (baby only)
        if pet.stage == LifeStage::Baby && pet.stats.happiness.value() < 60 {
            possible_events.push((BadDream, 0.25));
        }

        // Learned trick (child/teen/adult with high happiness)
        if matches!(
            pet.stage,
            LifeStage::Child | LifeStage::Teen | LifeStage::Adult
        ) && pet.stats.happiness.value() > 70
        {
            possible_events.push((LearnedTrick, 0.1));
        }

        // Happy moment (when stats are good)
        if pet.stats.happiness.value() > 60
            && pet.stats.health.value() > 70
            && pet.stats.energy.value() > 50
        {
            possible_events.push((HappyMoment, 0.1));
        }

        // Lonely (low happiness)
        if pet.stats.happiness.value() < 30 {
            possible_events.push((Lonely, 0.3));
        }

        // Asking for food (low hunger)
        if pet.stats.hunger.value() < 30 && pet.stage != LifeStage::Egg {
            possible_events.push((AskingForFood, 0.4));
        }

        // Weighted random selection
        let total_weight: f32 = possible_events.iter().map(|(_, w)| w).sum();
        if total_weight == 0.0 {
            return None;
        }

        let mut random = random_float() * total_weight;

        for (event, weight) in possible_events {
            random -= weight;
            if random <= 0.0 {
                return Some(event);
            }
        }

        // Fallback - should rarely reach here due to floating point
        None
    }

    /// Apply effects of an event to the pet
    fn apply_event_effects(&self, event_type: EventType, pet: &mut Pet) {
        use EventType::*;

        match event_type {
            MadeMess => {
                pet.stats.hygiene.sub(20);
            }
            FoundTreasure => {
                pet.stats.happiness.add(10);
            }
            BadDream => {
                pet.stats.happiness.sub(10);
                // Wake up baby if sleeping
                if pet.state.is_sleeping() {
                    pet.state = PetState::Normal;
                }
            }
            LearnedTrick => {
                pet.stats.happiness.add(15);
            }
            HappyMoment => {
                pet.stats.happiness.add(20);
                pet.stats.energy.add(5);
            }
            Lonely => {
                pet.stats.happiness.sub(5);
            }
            AskingForFood => {
                // No stat change, just a message
            }
            Evolved => {
                // Evolution handled separately
            }
        }
    }

    /// Clear the pending event (call after displaying)
    pub fn clear_pending(&mut self) {
        self.pending_event = None;
    }

    /// Get recent events as formatted strings
    pub fn recent_events(&self, count: usize) -> Vec<&GameEvent> {
        self.event_history.iter().rev().take(count).collect()
    }
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_message() {
        let event = EventType::MadeMess;
        let msg = event.message("Fluffy");
        assert!(msg.contains("Fluffy"));
        assert!(msg.contains("mess"));
    }

    #[test]
    fn test_event_history_limit() {
        let mut system = EventSystem::new();
        system.max_history = 3;

        // Create a dummy pet for testing
        let mut pet = Pet::new("Test");

        // Add 5 events manually
        for i in 0..5 {
            system.event_history.push(GameEvent {
                event_type: EventType::FoundTreasure,
                timestamp: Instant::now(),
                message: format!("Event {}", i),
            });
        }

        // Should only keep 3
        assert_eq!(system.event_history.len(), 5); // Before trimming

        // Trim manually
        while system.event_history.len() > system.max_history {
            system.event_history.remove(0);
        }

        assert_eq!(system.event_history.len(), 3);
    }
}

// Need to import Pet for tests
use crate::pet::Pet as _;
