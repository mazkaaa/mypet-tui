use std::collections::HashMap;
use std::sync::Arc;

use ratatui::style::Color;

use super::frame::AnimationFrame;
use super::types::AnimationType;

pub struct FrameCache {
    cache: HashMap<AnimationType, Arc<Vec<AnimationFrame>>>,
}

impl FrameCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();
        Self::load_builtin(&mut cache);
        Self { cache }
    }

    pub fn load(&self, anim_type: AnimationType) -> Arc<Vec<AnimationFrame>> {
        self.cache
            .get(&anim_type)
            .cloned()
            .unwrap_or_else(|| Arc::new(vec![Self::fallback_frame()]))
    }

    fn load_builtin(cache: &mut HashMap<AnimationType, Arc<Vec<AnimationFrame>>>) {
        cache.insert(
            AnimationType::IdleNeutral,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( o.o ) ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(500),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( -.- ) ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( o.o ) ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(500),
            ]),
        );

        cache.insert(
            AnimationType::IdleHappy,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ ) ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(400),
                AnimationFrame::new(vec![
                    "  \\   /  ".to_string(),
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ ) ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ ) ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(400),
            ]),
        );

        cache.insert(
            AnimationType::IdleSad,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ;.; ) ".to_string(),
                    "  > ~ <  ".to_string(),
                ])
                .with_duration(600),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ;.; ) ".to_string(),
                    "  > ~ <  ".to_string(),
                ])
                .with_duration(600),
            ]),
        );

        cache.insert(
            AnimationType::IdleSleeping,
            Arc::new(vec![AnimationFrame::new(vec![
                "  /\\_/\\  ".to_string(),
                " ( -.- ) ".to_string(),
                "  > ^ <  ".to_string(),
            ])
            .with_duration(800)
            .with_color(Color::DarkGray)]),
        );

        cache.insert(
            AnimationType::ActionEating,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  #".to_string(),
                    " ( o.o )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/#  ".to_string(),
                    " ( Oo  )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " (  -  )* ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )* ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
            ]),
        );

        cache.insert(
            AnimationType::ActionPlaying,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    " /  >  \\ ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "   /\\_/\\  ".to_string(),
                    "  ( ^.^ )  ".to_string(),
                    "   >  <   ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    " \\  <  / ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    " /\\_/\\   ".to_string(),
                    "( ^.^ )   ".to_string(),
                    "  < >    ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
            ]),
        );

        cache.insert(
            AnimationType::ActionCleaning,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( o.o )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\ * ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "* /\\_/\\ *".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    "* ( ^.^ )*".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
            ]),
        );

        cache.insert(
            AnimationType::ActionSleeping,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( -.- )  ".to_string(),
                    "  > ^ <  ".to_string(),
                    "  zZz    ".to_string(),
                ])
                .with_duration(400),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( -.- )  ".to_string(),
                    "  > ^ <  ".to_string(),
                    "   Zz    ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( -.- )  ".to_string(),
                    "  > ^ <  ".to_string(),
                    "    z    ".to_string(),
                ])
                .with_duration(300),
            ]),
        );

        cache.insert(
            AnimationType::ActionMedicine,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ;.; )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\ + ".to_string(),
                    " ( O.O )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )* ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(400),
            ]),
        );

        cache.insert(
            AnimationType::TransitionWakeUp,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( -.- )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( o.o )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(400),
            ]),
        );

        cache.insert(
            AnimationType::TransitionFallAsleep,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( o.o )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( -.- )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( -.- )  ".to_string(),
                    "  > ~ <  ".to_string(),
                    "  zZz    ".to_string(),
                ])
                .with_duration(400),
            ]),
        );

        cache.insert(
            AnimationType::TransitionEvolve,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( o.o )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(500),
                AnimationFrame::new(vec![
                    " */\\_/\\*  ".to_string(),
                    " *( o.o )* ".to_string(),
                    " * > ^ < * ".to_string(),
                ])
                .with_duration(500),
                AnimationFrame::new(vec![
                    "#*/\\_/\\*# ".to_string(),
                    "#*( o.o )*#".to_string(),
                    "#* > ^ < *#".to_string(),
                ])
                .with_duration(1000),
                AnimationFrame::new(vec![
                    " *#/\\_/\\#* ".to_string(),
                    "#*( ^.^ )*#".to_string(),
                    " *# > ^ < #*".to_string(),
                ])
                .with_duration(1000),
            ]),
        );

        cache.insert(
            AnimationType::TransitionGetSick,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( o.o )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\ ~ ".to_string(),
                    " ( ;.; )  ".to_string(),
                    "  > ~ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\ ~ ".to_string(),
                    " ( +.+ )  ".to_string(),
                    "  > ~ <  ".to_string(),
                ])
                .with_duration(400),
            ]),
        );

        cache.insert(
            AnimationType::TransitionHeal,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\ + ".to_string(),
                    " ( ;.; )  ".to_string(),
                    "  > ~ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\ *  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(300),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(400),
            ]),
        );

        cache.insert(
            AnimationType::TransitionDie,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( x.x )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(500),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( -.- )  ".to_string(),
                    "  > x <  ".to_string(),
                ])
                .with_duration(500),
                AnimationFrame::new(vec![
                    "   x_x    ".to_string(),
                    "   ---    ".to_string(),
                    "          ".to_string(),
                ])
                .with_duration(1000),
            ]),
        );

        cache.insert(
            AnimationType::MoodHappy,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )  ".to_string(),
                    "  > ^ <  ".to_string(),
                    " *   *  ".to_string(),
                ])
                .with_duration(500),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ^.^ )* ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(500),
            ]),
        );

        cache.insert(
            AnimationType::MoodExcited,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( *.* )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(150),
                AnimationFrame::new(vec![
                    "*  /\\_/\\*  ".to_string(),
                    " ( *.* )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(150),
            ]),
        );

        cache.insert(
            AnimationType::MoodSad,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ;.; )  ".to_string(),
                    "  > ~ <  ".to_string(),
                    " ~   ~  ".to_string(),
                ])
                .with_duration(600),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( ;.; )~ ".to_string(),
                    "  > ~ <  ".to_string(),
                ])
                .with_duration(600),
            ]),
        );

        cache.insert(
            AnimationType::MoodAngry,
            Arc::new(vec![
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( >.< )  ".to_string(),
                    "  > ^ <  ".to_string(),
                ])
                .with_duration(200),
                AnimationFrame::new(vec![
                    "  /\\_/\\  ".to_string(),
                    " ( >.< )  ".to_string(),
                    "  > n <  ".to_string(),
                ])
                .with_duration(200),
            ]),
        );
    }

    fn fallback_frame() -> AnimationFrame {
        AnimationFrame::new(vec![
            "  /\\_/\\  ".to_string(),
            " ( ?.? )  ".to_string(),
            "  > ^ <  ".to_string(),
        ])
    }
}

impl Default for FrameCache {
    fn default() -> Self {
        Self::new()
    }
}
