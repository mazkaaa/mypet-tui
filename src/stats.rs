//! Stats system with bounded values

use std::ops::{Add, Sub};

/// A bounded value that clamps between MIN and MAX
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatValue {
    value: u8,
}

impl StatValue {
    /// Minimum value (0)
    pub const MIN: u8 = 0;
    /// Maximum value (100)
    pub const MAX: u8 = 100;

    /// Create a new StatValue, clamped to valid range
    pub fn new(value: u8) -> Self {
        Self {
            value: value.clamp(Self::MIN, Self::MAX),
        }
    }

    /// Get the current value
    pub fn value(&self) -> u8 {
        self.value
    }

    /// Add to the value, clamping at MAX
    pub fn add(&mut self, amount: u8) {
        self.value = (self.value.saturating_add(amount)).min(Self::MAX);
    }

    /// Subtract from the value, clamping at MIN
    pub fn sub(&mut self, amount: u8) {
        self.value = self.value.saturating_sub(amount);
    }

    /// Set the value directly, clamped to range
    pub fn set(&mut self, value: u8) {
        self.value = value.clamp(Self::MIN, Self::MAX);
    }

    /// Check if value is at maximum
    pub fn is_max(&self) -> bool {
        self.value == Self::MAX
    }

    /// Check if value is at minimum
    pub fn is_min(&self) -> bool {
        self.value == Self::MIN
    }

    /// Get value as percentage (0-100)
    pub fn percentage(&self) -> f32 {
        (self.value as f32 / Self::MAX as f32) * 100.0
    }
}

impl Default for StatValue {
    fn default() -> Self {
        Self::new(50)
    }
}

/// All pet stats
#[derive(Debug, Clone)]
pub struct Stats {
    /// Hunger (0-100), 0 = starving, 100 = full
    pub hunger: StatValue,
    /// Happiness (0-100), 0 = depressed, 100 = ecstatic
    pub happiness: StatValue,
    /// Energy (0-100), 0 = exhausted, 100 = fully rested
    pub energy: StatValue,
    /// Health (0-100), 0 = dead, 100 = perfect health
    pub health: StatValue,
    /// Hygiene (0-100), 0 = filthy, 100 = spotless
    pub hygiene: StatValue,
}

impl Stats {
    /// Create new stats with default values
    pub fn new() -> Self {
        Self {
            hunger: StatValue::new(50),
            happiness: StatValue::new(50),
            energy: StatValue::new(50),
            health: StatValue::new(100),
            hygiene: StatValue::new(50),
        }
    }

    /// Check if pet is starving (hunger at 0)
    pub fn is_starving(&self) -> bool {
        self.hunger.is_min()
    }

    /// Check if pet is depressed (happiness at 0)
    pub fn is_depressed(&self) -> bool {
        self.happiness.is_min()
    }

    /// Check if pet is exhausted (energy at 0)
    pub fn is_exhausted(&self) -> bool {
        self.energy.is_min()
    }

    /// Check if pet is critically unhealthy (health below 20)
    pub fn is_critical(&self) -> bool {
        self.health.value() < 20
    }

    /// Check if pet is filthy (hygiene at 0)
    pub fn is_filthy(&self) -> bool {
        self.hygiene.is_min()
    }

    /// Apply natural decay over time
    pub fn decay(&mut self) {
        // Hunger increases over time (pet gets hungrier)
        self.hunger.sub(1);
        // Happiness slowly decreases without interaction
        self.happiness.sub(1);
        // Energy slowly decreases
        self.energy.sub(1);
        // Hygiene decreases over time
        self.hygiene.sub(1);

        // Health is affected by other stats
        if self.is_starving() || self.is_depressed() || self.is_filthy() {
            self.health.sub(1);
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_value_clamps_at_max() {
        let mut stat = StatValue::new(50);
        stat.add(100);
        assert_eq!(stat.value(), 100);
    }

    #[test]
    fn stat_value_clamps_at_min() {
        let mut stat = StatValue::new(50);
        stat.sub(100);
        assert_eq!(stat.value(), 0);
    }

    #[test]
    fn stat_value_new_clamps_input() {
        let stat = StatValue::new(150);
        assert_eq!(stat.value(), 100);

        let stat = StatValue::new(0);
        assert_eq!(stat.value(), 0);
    }

    #[test]
    fn stats_decay_reduces_values() {
        let mut stats = Stats::new();
        let initial_hunger = stats.hunger.value();

        stats.decay();

        assert!(stats.hunger.value() < initial_hunger);
    }
}
