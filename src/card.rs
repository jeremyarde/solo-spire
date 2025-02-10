pub mod card {
    use bevy::prelude::*;

    // #[derive(Component, Clone)]
    // struct Damage(usize);

    #[derive(Component, Clone)]
    pub enum CardEffect {
        DirectDamage(usize),
        DamageOverTime {
            damage: usize,
            duration: f32,
            frequency: f32,
        },
        Stun {
            duration: f32,
        },
        Heal(usize),
    }

    #[derive(Component, Clone)]
    pub enum ActiveEffect {
        DirectDamage(usize),
        DamageOverTime {
            damage: usize,
            duration: Timer,
            frequency: Timer,
        },
        Stun {
            duration: Timer,
        },
        Heal(usize),
    }

    // pub struct Effect {
    //     effect: CardEffect,
    //     cooldown: Timer,
    // }

    #[derive(Component, Clone)]
    pub struct Effects {
        pub effects: Vec<ActiveEffect>,
    }
}
