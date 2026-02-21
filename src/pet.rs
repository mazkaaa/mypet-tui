//! Pet struct and logic

use std::time::{Duration, Instant};

use crate::stats::{StatValue, Stats};

/// Life stages of a pet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeStage {
    /// Egg stage (first 30 seconds)
    Egg,
    /// Baby stage (30s - 5 minutes)
    Baby,
    /// Child stage (5 - 15 minutes)
    Child,
    /// Teen stage (15 - 30 minutes)
    Teen,
    /// Adult stage (30+ minutes)
    Adult,
}

impl LifeStage {
    /// Get the next life stage
    pub fn next(self) -> Option<Self> {
        match self {
            LifeStage::Egg => Some(LifeStage::Baby),
            LifeStage::Baby => Some(LifeStage::Child),
            LifeStage::Child => Some(LifeStage::Teen),
            LifeStage::Teen => Some(LifeStage::Adult),
            LifeStage::Adult => None,
        }
    }

    /// Get display name for the stage
    pub fn display_name(self) -> &'static str {
        match self {
            LifeStage::Egg => "Egg",
            LifeStage::Baby => "Baby",
            LifeStage::Child => "Child",
            LifeStage::Teen => "Teen",
            LifeStage::Adult => "Adult",
        }
    }

    /// Get ASCII art for the stage
    pub fn ascii_art(self) -> &'static str {
        match self {
            LifeStage::Egg => {
                r#"
        , - ~ ~ ~ - ,
    , '               ' ,
  ,                       ,
 ,                         ,
 ,                         ,
  ,                       ,
    ,                  , '
      ' - , _ _ _ ,  '
"#
            }
            LifeStage::Baby => {
                r#"
       (◕‿◕)
        /|\
         |
        / \
"#
            }
            LifeStage::Child => {
                r#"
      \\(◕‿◕)/
         | |
        /   \
"#
            }
            LifeStage::Teen => {
                r#"
       /\\_/\\
      ( ◕‿◕ )
       > ^ <
      /     \
"#
            }
            LifeStage::Adult => {
                r#"
        /\\_/\\
       ( o.o )
        > ^ <
       /|   |\
        |   |
       /     \
"#
            }
        }
    }
}

/// Current state of the pet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetState {
    /// Normal active state
    Normal,
    /// Pet is sleeping
    Sleeping { since: Instant },
    /// Pet is sick
    Sick { since: Instant },
    /// Pet has died :(
    Dead,
}

impl PetState {
    /// Check if pet can perform actions
    pub fn can_act(self) -> bool {
        matches!(self, PetState::Normal) || matches!(self, PetState::Sick { .. })
    }

    /// Check if pet is sleeping
    pub fn is_sleeping(self) -> bool {
        matches!(self, PetState::Sleeping { .. })
    }

    /// Check if pet is sick
    pub fn is_sick(self) -> bool {
        matches!(self, PetState::Sick { .. })
    }

    /// Check if pet is alive
    pub fn is_alive(self) -> bool {
        !matches!(self, PetState::Dead)
    }
}

/// Stats specific to Egg stage
#[derive(Debug, Clone)]
pub struct EggStats {
    /// Incubation progress (0-100), time-based
    pub incubation_progress: StatValue,
    /// Warmth level (0-100), player-controlled
    pub warmth_level: StatValue,
    /// Egg health (0-100), only relevant when cold
    pub health: StatValue,
    /// Whether the egg died (failed to hatch)
    pub is_dead: bool,
}

impl EggStats {
    /// Create new egg stats
    pub fn new() -> Self {
        Self {
            incubation_progress: StatValue::new(0),
            warmth_level: StatValue::new(20), // Start a bit cold
            health: StatValue::new(100),
            is_dead: false,
        }
    }
}

impl Default for EggStats {
    fn default() -> Self {
        Self::new()
    }
}

/// The main Pet struct
#[derive(Debug, Clone)]
pub struct Pet {
    /// Pet's name
    pub name: String,
    /// Current life stage
    pub stage: LifeStage,
    /// Current state
    pub state: PetState,
    /// All stats (used after hatching)
    pub stats: Stats,
    /// When the pet was born
    pub birth_time: Instant,
    /// Total age in seconds
    pub age_seconds: u64,
    /// Last time stats were decayed
    last_decay: Instant,
    /// Egg-specific stats (only used during Egg stage)
    pub egg_stats: Option<EggStats>,
}

impl Pet {
    /// Create a new pet with the given name
    pub fn new(name: impl Into<String>) -> Self {
        let now = Instant::now();
        Self {
            name: name.into(),
            stage: LifeStage::Egg,
            state: PetState::Normal,
            stats: Stats::new(),
            birth_time: now,
            age_seconds: 0,
            last_decay: now,
            egg_stats: Some(EggStats::new()),
        }
    }

    /// Check if egg is dead (failed to hatch)
    pub fn is_egg_dead(&self) -> bool {
        match &self.egg_stats {
            Some(egg) => egg.is_dead,
            None => false,
        }
    }

    /// Get egg warmth level (0-100)
    pub fn get_warmth(&self) -> u8 {
        match &self.egg_stats {
            Some(egg) => egg.warmth_level.value(),
            None => 0,
        }
    }

    /// Get egg incubation progress (0-100)
    pub fn get_incubation(&self) -> u8 {
        match &self.egg_stats {
            Some(egg) => egg.incubation_progress.value(),
            None => 100,
        }
    }

    /// Get egg health (0-100)
    pub fn get_egg_health(&self) -> u8 {
        match &self.egg_stats {
            Some(egg) => egg.health.value(),
            None => 100,
        }
    }

    /// Get the age formatted nicely
    pub fn age_formatted(&self) -> String {
        let seconds = self.age_seconds % 60;
        let minutes = (self.age_seconds / 60) % 60;
        let hours = self.age_seconds / 3600;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Update the pet (call every frame/tick)
    pub fn update(&mut self, delta_time: Duration) {
        // Update age
        self.age_seconds += delta_time.as_secs();

        // Handle Egg stage separately
        if self.stage == LifeStage::Egg {
            self.update_egg();
            return;
        }

        // For hatched pets
        if !self.state.is_alive() {
            return;
        }

        // Update life stage based on age
        self.update_life_stage();

        // Apply stat decay every 5 seconds
        if self.last_decay.elapsed() >= Duration::from_secs(5) {
            self.stats.decay();
            self.last_decay = Instant::now();

            // Check for death
            if self.stats.health.value() == 0 {
                self.state = PetState::Dead;
            }

            // Check for sickness if hygiene is very low
            if self.stats.hygiene.value() < 10 && matches!(self.state, PetState::Normal) {
                self.state = PetState::Sick {
                    since: Instant::now(),
                };
            }
        }
    }

    /// Update egg mechanics
    fn update_egg(&mut self) {
        if let Some(ref mut egg) = self.egg_stats {
            // Check every 5 seconds
            if self.last_decay.elapsed() >= Duration::from_secs(5) {
                self.last_decay = Instant::now();

                // Incubation progress increases over time (30 seconds total = 100%)
                // Every 5 seconds = ~16.67% progress
                egg.incubation_progress.add(17);

                // Warmth decays slowly (-3 every 5 seconds)
                egg.warmth_level.sub(3);

                // Health mechanics based on warmth
                if egg.warmth_level.value() < 30 {
                    // Egg is too cold - health drops
                    egg.health.sub(10);
                } else {
                    // Egg is warm enough - health recovers slowly
                    egg.health.add(5);
                }

                // Check if egg died
                if egg.health.value() == 0 {
                    egg.is_dead = true;
                }

                // Check if ready to hatch
                if egg.incubation_progress.is_max() && !egg.is_dead {
                    self.hatch_egg();
                }
            }
        }
    }

    /// Hatch the egg into a baby
    fn hatch_egg(&mut self) {
        if let Some(ref egg) = self.egg_stats {
            // Apply warmth bonuses to baby stats
            let warmth = egg.warmth_level.value();

            if warmth >= 70 {
                // High warmth = strong baby
                self.stats.health.add(20);
                self.stats.happiness.add(20);
                self.stats.energy.add(20);
                self.stats.hunger.add(20);
                self.stats.hygiene.add(20);
            } else if warmth < 40 {
                // Low warmth = weak baby
                self.stats.health.sub(10);
                self.stats.happiness.sub(10);
                self.stats.energy.sub(10);
            }
            // Medium warmth = normal stats (no change)
        }

        // Hatch!
        self.stage = LifeStage::Baby;
        self.egg_stats = None; // No longer needed
    }

    /// Warm the egg (only available in Egg stage)
    pub fn warm(&mut self) -> Result<(), &'static str> {
        if self.stage != LifeStage::Egg {
            return Err("The pet has already hatched!");
        }

        if let Some(ref mut egg) = self.egg_stats {
            if egg.warmth_level.value() >= 100 {
                return Err("The egg is warm enough!");
            }

            egg.warmth_level.add(10);

            // Warming increases egg health
            egg.health.add(5);
        }

        Ok(())
    }

    /// Restart with a new egg (game over)
    pub fn restart(&mut self) {
        *self = Self::new(&self.name);
    }

    /// Feed the pet
    pub fn feed(&mut self) -> Result<(), &'static str> {
        if self.stage == LifeStage::Egg {
            return Err("Can't feed an egg! Try warming it instead.");
        }

        if !self.state.can_act() {
            return Err("Pet cannot eat right now");
        }

        self.stats.hunger.add(25);
        self.stats.energy.sub(5); // Eating takes some energy
        Ok(())
    }

    /// Play with the pet
    pub fn play(&mut self) -> Result<(), &'static str> {
        if self.stage == LifeStage::Egg {
            return Err("Can't play with an egg!");
        }

        if !self.state.can_act() {
            return Err("Pet cannot play right now");
        }

        // Baby stage restrictions
        if self.stage == LifeStage::Baby {
            if self.stats.energy.value() < 30 {
                return Err("Baby is too tired. Let it sleep first!");
            }
            // Baby can't play for long
            self.stats.happiness.add(15); // Less happiness gain
            self.stats.energy.sub(20); // More energy cost
            self.stats.hunger.sub(10);
            return Ok(());
        }

        if self.stats.energy.value() < 20 {
            return Err("Pet is too tired to play");
        }

        self.stats.happiness.add(20);
        self.stats.energy.sub(15);
        self.stats.hunger.sub(10); // Playing makes hungry
        Ok(())
    }

    /// Clean the pet
    pub fn clean(&mut self) -> Result<(), &'static str> {
        if self.stage == LifeStage::Egg {
            return Err("Can't clean an egg!");
        }

        if !self.state.can_act() {
            return Err("Pet cannot be cleaned right now");
        }

        self.stats.hygiene = StatValue::new(100);

        // Cleaning can cure sickness
        if matches!(self.state, PetState::Sick { .. }) {
            self.state = PetState::Normal;
        }

        Ok(())
    }

    /// Put pet to sleep
    pub fn sleep(&mut self) -> Result<(), &'static str> {
        if self.stage == LifeStage::Egg {
            return Err("Eggs don't sleep! Try warming it.");
        }

        if !self.state.is_alive() {
            return Err("Pet is dead");
        }

        if matches!(self.state, PetState::Sleeping { .. }) {
            return Err("Pet is already sleeping");
        }

        self.state = PetState::Sleeping {
            since: Instant::now(),
        };
        Ok(())
    }

    /// Wake up the pet
    pub fn wake(&mut self) -> Result<(), &'static str> {
        match self.state {
            PetState::Sleeping { since } => {
                let sleep_duration = since.elapsed().as_secs();
                // Regenerate energy based on sleep duration
                let base_gain = ((sleep_duration as f32 / 30.0) * 100.0).min(100.0) as u8;

                // Baby stage gets more energy from sleep
                let energy_gain = if self.stage == LifeStage::Baby {
                    (base_gain as f32 * 1.5).min(100.0) as u8
                } else {
                    base_gain
                };

                self.stats.energy.add(energy_gain);
                self.state = PetState::Normal;
                Ok(())
            }
            _ => Err("Pet is not sleeping"),
        }
    }

    /// Give medicine to the pet
    pub fn give_medicine(&mut self) -> Result<(), &'static str> {
        if matches!(self.state, PetState::Sick { .. }) {
            self.state = PetState::Normal;
            self.stats.health.add(20);
            Ok(())
        } else {
            Err("Pet is not sick")
        }
    }

    /// Update life stage based on age
    pub fn update_life_stage(&mut self) {
        if self.stage == LifeStage::Egg {
            return;
        }

        let age_minutes = self.age_seconds / 60;

        let new_stage = match self.stage {
            LifeStage::Egg => LifeStage::Egg,
            LifeStage::Baby if age_minutes >= 5 => LifeStage::Child,
            LifeStage::Child if age_minutes >= 15 => LifeStage::Teen,
            LifeStage::Teen if age_minutes >= 30 => LifeStage::Adult,
            _ => return,
        };

        if new_stage != self.stage {
            self.stage = new_stage;
        }
    }

    /// Get a status message
    pub fn status_message(&self) -> String {
        if !self.state.is_alive() {
            return format!("{} has passed away...", self.name);
        }

        // Egg stage messages based on warmth level
        if self.stage == LifeStage::Egg {
            if let Some(ref egg) = self.egg_stats {
                if egg.is_dead {
                    return format!("The egg failed to hatch... It was too cold.");
                }

                let warmth = egg.warmth_level.value();
                let health = egg.health.value();

                // Critical health warning
                if health < 30 {
                    return format!("⚠ CRITICAL: The egg is dying! Warm it NOW!");
                }

                // Messages based on warmth level
                return match warmth {
                    0..=20 => format!("⚠ The egg is FREEZING! Warm it quickly or it will die!"),
                    21..=40 => format!("The egg feels cold... Try pressing [W] to warm it!"),
                    41..=60 => format!("The egg is getting warmer..."),
                    61..=80 => format!("The egg is cozy and warm!"),
                    81..=95 => format!("The egg is very warm! The baby will be healthy!"),
                    _ => format!("🎉 The egg is ready to hatch any moment!"),
                };
            }
        }

        match self.state {
            PetState::Sleeping { .. } => format!("{} is sleeping peacefully", self.name),
            PetState::Sick { .. } => format!("{} is not feeling well", self.name),
            _ => {
                if self.stats.is_starving() {
                    format!("{} is very hungry!", self.name)
                } else if self.stats.is_depressed() {
                    format!("{} seems sad...", self.name)
                } else if self.stats.is_exhausted() {
                    format!("{} is exhausted", self.name)
                } else if self.stats.is_filthy() {
                    format!("{} needs a bath", self.name)
                } else if self.stage == LifeStage::Baby {
                    format!("{} is a cute baby!", self.name)
                } else {
                    format!("{} is doing well!", self.name)
                }
            }
        }
    }
}

impl Default for Pet {
    fn default() -> Self {
        Self::new("Fluffy")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pet_starts_as_egg() {
        let pet = Pet::new("Test");
        assert_eq!(pet.stage, LifeStage::Egg);
    }

    #[test]
    fn pet_ages_correctly() {
        let mut pet = Pet::new("Test");
        // Simulate hatching by setting age to after hatching and clearing egg_stats
        pet.age_seconds = 31; // Just over 30s
        pet.stage = LifeStage::Baby; // Skip egg stage
        pet.update_life_stage();
        assert_eq!(pet.stage, LifeStage::Baby);

        // Now test transition to Child (5 minutes = 300 seconds)
        pet.age_seconds = 301;
        pet.update_life_stage();
        assert_eq!(pet.stage, LifeStage::Child);
    }

    #[test]
    fn feeding_increases_hunger() {
        let mut pet = Pet::new("Test");
        pet.stage = LifeStage::Baby; // Skip egg stage
        pet.stats.hunger = StatValue::new(30);
        pet.feed().unwrap();
        assert!(pet.stats.hunger.value() > 30);
    }

    #[test]
    fn sleeping_pet_cannot_eat() {
        let mut pet = Pet::new("Test");
        pet.state = PetState::Sleeping {
            since: Instant::now(),
        };
        assert!(pet.feed().is_err());
    }
}
