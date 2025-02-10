pub mod card {

    use std::mem;

    use bevy::{prelude::*, reflect::GetTypeRegistration};

    // #[derive(Component, Clone)]
    // struct Damage(usize);

    #[derive(Component, Clone)]
    pub enum CardEffect {
        DirectDamage(i32),
        DamageOverTime {
            damage: i32,
            duration: f32,
            frequency: f32,
        },
        Stun {
            duration: f32,
        },
        Heal(i32),
    }

    pub enum StatusEffect {
        Bleed,
        Poison,
        Burn,
        Freeze,
        Shock,
        Stun,
        Silence,
        Disarm,
    }

    pub enum Element {
        Fire,
        Water,
        Earth,
        Air,
        Light,
    }

    #[derive(Component, Clone, Reflect)]
    pub enum ActiveEffect {
        DirectDamage(i32),
        DamageOverTime {
            damage: i32,
            duration: Timer,
            frequency: Timer,
        },
        Stun {
            duration: Timer,
        },
        Heal(i32),
    }

    // pub struct Effect {
    //     effect: CardEffect,
    //     cooldown: Timer,
    // }

    #[derive(Component, Clone, Reflect)]
    pub struct Effects {
        pub effects: Vec<ActiveEffect>,
    }

    impl CardEffect {
        pub fn get_random_effect() -> CardEffect {
            use rand::random_range;

            let effect_type = random_range(0..4);
            match effect_type {
                0 => CardEffect::DirectDamage(random_range(5..20)),
                1 => CardEffect::DamageOverTime {
                    damage: random_range(2..8),
                    duration: random_range(2.0..5.0),
                    frequency: random_range(0.3..1.0),
                },
                2 => CardEffect::Stun {
                    duration: random_range(1.0..3.0),
                },
                _ => CardEffect::Heal(random_range(5..15)),
            }
        }

        pub fn get_sprite_path(&self) -> String {
            match self {
                CardEffect::DirectDamage(_) => "direct.png".to_string(),
                CardEffect::DamageOverTime { .. } => "dot.png".to_string(),
                CardEffect::Stun { .. } => "stun.png".to_string(),
                CardEffect::Heal(_) => "heal.png".to_string(),
            }
        }
    }
}
