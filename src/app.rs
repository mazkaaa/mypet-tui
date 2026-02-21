//! Application state and main loop logic

use std::time::Instant;

use crate::animation::types::AnimationType;
use crate::events::EventSystem;
use crate::pet::{LifeStage, Pet, PetState};
use crate::widgets::AnimatedPet;

/// Game state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// Normal gameplay
    Playing,
    /// Egg died - game over
    GameOver,
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Whether the application should quit
    pub should_quit: bool,
    /// Current game state
    pub game_state: GameState,
    /// The pet
    pub pet: Pet,
    /// Last update time
    last_update: Instant,
    /// Status message
    pub status_message: String,
    /// Event system for random occurrences
    pub event_system: EventSystem,
    /// Animation system
    pub animated_pet: AnimatedPet,
    /// Last animation update time
    last_animation_update: Instant,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        let pet = Pet::new("Fluffy");
        let status = pet.status_message();

        Self {
            should_quit: false,
            game_state: GameState::Playing,
            pet,
            last_update: Instant::now(),
            status_message: status,
            event_system: EventSystem::new(),
            animated_pet: AnimatedPet::new(),
            last_animation_update: Instant::now(),
        }
    }

    /// Handle tick event (called periodically)
    pub fn tick(&mut self) {
        // Update animation at 10 FPS
        self.update_animation();

        // Check for game over
        if self.game_state == GameState::GameOver {
            return;
        }

        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.last_update = now;

        // Update the pet
        self.pet.update(delta);

        // Update animation based on pet state
        self.update_pet_animation();

        // Check if egg died
        if self.pet.is_egg_dead() {
            self.game_state = GameState::GameOver;
            self.status_message = "The egg failed to hatch... Game Over!".to_string();
            self.animated_pet.trigger(AnimationType::TransitionDie);
            return;
        }

        // Update event system (only for hatched pets)
        if self.pet.stage != LifeStage::Egg {
            self.event_system.update(&mut self.pet, delta);

            // Check for pending events and display them
            if let Some(event) = self.event_system.pending_event.take() {
                self.status_message = event.message;
                return;
            }
        }

        // Update status message
        self.status_message = self.pet.status_message();
    }

    /// Update animation system (called at 10 FPS)
    pub fn update_animation(&mut self) {
        if self.last_animation_update.elapsed().as_millis() >= 100 {
            self.animated_pet.update();
            self.last_animation_update = Instant::now();
        }
    }

    /// Update animation based on pet state
    fn update_pet_animation(&mut self) {
        use crate::pet::PetState;

        match self.pet.state {
            PetState::Normal => {
                // Check happiness level for idle animation
                if self.pet.stats.happiness.value() < 30 {
                    self.animated_pet.set_idle_sad();
                } else if self.pet.stats.happiness.value() > 70 {
                    self.animated_pet.set_idle_happy();
                } else {
                    self.animated_pet.set_idle();
                }
            }
            PetState::Sleeping { .. } => {
                self.animated_pet.set_idle_sleeping();
            }
            PetState::Sick { .. } => {
                self.animated_pet.trigger(AnimationType::TransitionGetSick);
            }
            PetState::Dead => {
                self.animated_pet.trigger(AnimationType::TransitionDie);
            }
        }
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Restart the game (only works in Game Over state)
    pub fn restart(&mut self) {
        if self.game_state == GameState::GameOver {
            let name = self.pet.name.clone();
            self.pet = Pet::new(&name);
            self.game_state = GameState::Playing;
            self.event_system = EventSystem::new();
            self.status_message = self.pet.status_message();
            self.last_update = Instant::now();
            self.animated_pet = AnimatedPet::new();
        }
    }

    /// Warm the egg (only in Egg stage)
    pub fn warm_egg(&mut self) {
        if self.game_state == GameState::GameOver {
            return;
        }

        match self.pet.warm() {
            Ok(()) => {
                let warmth = self.pet.get_warmth();
                self.status_message = format!("You warmed the egg! Warmth: {}%", warmth);
            }
            Err(msg) => self.status_message = msg.to_string(),
        }
    }

    /// Feed the pet
    pub fn feed_pet(&mut self) {
        if self.game_state == GameState::GameOver {
            return;
        }

        match self.pet.feed() {
            Ok(()) => {
                self.status_message = format!("You fed {}!", self.pet.name);
                self.animated_pet.trigger(AnimationType::ActionEating);
            }
            Err(msg) => self.status_message = msg.to_string(),
        }
    }

    /// Play with the pet
    pub fn play_with_pet(&mut self) {
        if self.game_state == GameState::GameOver {
            return;
        }

        match self.pet.play() {
            Ok(()) => {
                self.status_message = format!("You played with {}!", self.pet.name);
                self.animated_pet.trigger(AnimationType::ActionPlaying);
            }
            Err(msg) => self.status_message = msg.to_string(),
        }
    }

    /// Clean the pet
    pub fn clean_pet(&mut self) {
        if self.game_state == GameState::GameOver {
            return;
        }

        match self.pet.clean() {
            Ok(()) => {
                self.status_message = format!("You cleaned {}!", self.pet.name);
                self.animated_pet.trigger(AnimationType::ActionCleaning);
            }
            Err(msg) => self.status_message = msg.to_string(),
        }
    }

    /// Toggle sleep/wake
    pub fn toggle_sleep(&mut self) {
        if self.game_state == GameState::GameOver {
            return;
        }

        match self.pet.state {
            PetState::Sleeping { .. } => match self.pet.wake() {
                Ok(()) => {
                    self.status_message = format!("{} woke up!", self.pet.name);
                    self.animated_pet.trigger(AnimationType::TransitionWakeUp);
                }
                Err(msg) => self.status_message = msg.to_string(),
            },
            _ => match self.pet.sleep() {
                Ok(()) => {
                    self.status_message = format!("{} went to sleep!", self.pet.name);
                    self.animated_pet.trigger(AnimationType::ActionSleeping);
                }
                Err(msg) => self.status_message = msg.to_string(),
            },
        }
    }

    /// Give medicine to the pet
    pub fn give_medicine(&mut self) {
        if self.game_state == GameState::GameOver {
            return;
        }

        match self.pet.give_medicine() {
            Ok(()) => {
                self.status_message = format!("You gave {} medicine!", self.pet.name);
                self.animated_pet.trigger(AnimationType::ActionMedicine);
            }
            Err(msg) => self.status_message = msg.to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
