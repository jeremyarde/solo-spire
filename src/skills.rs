pub mod skills {
    use bevy::prelude::*;

    #[derive(Component, Clone, Default)]
    pub struct Stats {
        pub strength: usize,
        pub agility: usize,
        pub stamina: usize,
        pub perception: usize,
        pub intelligence: usize,
    }

    #[derive(Component, Clone)]
    pub enum Class {
        Warrior,
        Rogue,
        Mage,
        Healer,
        Assassin,
    }
}
