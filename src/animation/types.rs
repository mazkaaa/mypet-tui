use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnimationPriority {
    Background = 0,
    Idle = 1,
    Mood = 2,
    Action = 3,
    Transition = 4,
    Critical = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationType {
    IdleNeutral,
    IdleHappy,
    IdleSad,
    IdleSleeping,

    MoodHappy,
    MoodExcited,
    MoodSad,
    MoodAngry,

    ActionEating,
    ActionPlaying,
    ActionCleaning,
    ActionSleeping,
    ActionMedicine,

    TransitionWakeUp,
    TransitionFallAsleep,
    TransitionEvolve,
    TransitionGetSick,
    TransitionHeal,
    TransitionDie,

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
            EffectHearts | EffectFood | EffectSparkles | EffectZzz | EffectSweat => {
                AnimationPriority::Background
            }
            IdleNeutral | IdleHappy | IdleSad | IdleSleeping => AnimationPriority::Idle,
            MoodHappy | MoodExcited | MoodSad | MoodAngry => AnimationPriority::Mood,
            ActionEating | ActionPlaying | ActionCleaning | ActionSleeping | ActionMedicine => {
                AnimationPriority::Action
            }
            TransitionWakeUp | TransitionFallAsleep | TransitionEvolve | TransitionGetSick
            | TransitionHeal | TransitionDie => AnimationPriority::Transition,
        }
    }

    pub fn is_infinite(&self) -> bool {
        matches!(
            self,
            AnimationType::IdleNeutral
                | AnimationType::IdleHappy
                | AnimationType::IdleSad
                | AnimationType::IdleSleeping
                | AnimationType::MoodHappy
                | AnimationType::MoodSad
        )
    }

    pub fn duration_ms(&self) -> u64 {
        use AnimationType::*;

        match self {
            IdleNeutral | IdleHappy | IdleSad | IdleSleeping => 0,

            ActionEating => 1500,
            ActionPlaying => 2000,
            ActionCleaning => 1000,
            ActionSleeping => 1000,
            ActionMedicine => 1000,

            TransitionWakeUp => 1000,
            TransitionFallAsleep => 1500,
            TransitionEvolve => 3000,
            TransitionGetSick => 1000,
            TransitionHeal => 1000,
            TransitionDie => 2000,

            EffectHearts | EffectFood | EffectSparkles | EffectZzz | EffectSweat => 2000,

            _ => 1000,
        }
    }
}
