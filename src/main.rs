//! Demonstrates picking for sprites and sprite atlases. The picking backend only tests against the
//! sprite bounds, so the sprite atlas can be picked by clicking on its transparent areas.

use bevy::{prelude::*, state::commands, ui::Interaction, window::WindowResolution};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use card::card::{ActiveEffect, CardEffect, Effects};
use rand::random_range;
use skills::skills::{Class, Stats};
use std::fmt::Debug;

mod card;
mod skills;

const MENU_ALPHA: f32 = 0.8;
const INVENTORY_ITEM_HEIGHT: f32 = 50.0;
const INVENTORY_VISIBLE_ITEMS: f32 = 8.0; // Number of items visible at once
const SCROLL_SPEED: f32 = 20.0;

#[derive(Resource, Debug, Default)]
struct GameConfig {
    screen_width: f32,
    screen_height: f32,
}

#[derive(Component)]
struct EnemyHealth(i32);

#[derive(Component)]
struct PlayerHealth(i32);

#[derive(Component)]
struct PlayerEntity;

#[derive(Component, Deref, DerefMut)]
struct CardAttackTimer(Timer);

pub const RED: Color = Color::srgb(1.0, 0.0, 0.0);
pub const YELLOW: Color = Color::srgb(1.0, 1.0, 0.0);
pub const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Battle,
    LootScreen,
    Menu,
    EndBattle,
}

#[derive(Component, Clone)]
struct LootItem {
    name: String,
    rarity: LootRarity,
}

#[derive(Component, Clone, Copy)]
enum LootRarity {
    Common,
    Rare,
    Epic,
}

impl LootRarity {
    fn get_color(&self) -> Color {
        match self {
            LootRarity::Common => Color::rgba(0.8, 0.8, 0.8, 0.7),
            LootRarity::Rare => Color::rgba(0.0, 0.5, 1.0, 0.7),
            LootRarity::Epic => Color::rgba(0.8, 0.0, 0.8, 0.7),
        }
    }

    fn get_text_color(&self) -> Color {
        match self {
            LootRarity::Common => Color::rgb(0.2, 0.2, 0.2),
            LootRarity::Rare => Color::WHITE,
            LootRarity::Epic => Color::rgb(1.0, 0.9, 0.0),
        }
    }
}

#[derive(Resource, Default)]
struct Inventory {
    items: Vec<LootItem>,
}

#[derive(Component)]
struct InventoryButton;

#[derive(Component)]
struct InventoryDisplay;

#[derive(Component)]
struct LootAllButton;

fn debug_display_state(state: Res<State<GameState>>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        println!("Current state: {:?}", state.get());
    }
    if input.just_pressed(KeyCode::KeyQ) {
        println!("Q pressed");
        // quit the game
        std::process::exit(0);
    }
}

#[derive(Component)]
struct EnemyEntity;

#[derive(Bundle)]
struct EnemyBundle {
    enemy: EnemyEntity,
    name: Name,
    sprite: Sprite,
    transform: Transform,
    enemy_health: EnemyHealth,
    stats: Stats,
    effects: Effects,
}

enum CardPosition {
    Top,
    Bottom,
}

fn on_enter_battle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_config: Res<GameConfig>,
    enemy_query: Query<Entity, With<EnemyEntity>>,
) {
    if let Ok(enemy) = enemy_query.get_single() {
        println!("Enemy already exists");
        return;
    }
    let enemybundle = spawn_new_enemy(asset_server.load("boss_bee.png"), &game_config);
    let enemyid = commands.spawn(enemybundle).id();

    let num_cards = random_range(1..4);
    commands.entity(enemyid).with_children(|parent| {
        for i in 0..num_cards {
            add_card(
                parent,
                // asset_server.load("player.png"),
                &asset_server,
                Vec2::splat(128.0 / 2.0),
                EnemyCard,
                (i, num_cards),
                CardPosition::Bottom,
            );
        }
        parent.spawn((
            Name::new("Enemy Health"),
            Text2d::new("100"),
            Transform::from_xyz(0.0, -20.0, 0.1),
            EnemyHealthText,
        ));
    });
}

fn spawn_new_enemy(image: Handle<Image>, game_config: &GameConfig) -> EnemyBundle {
    println!("Spawning new enemy");
    let sprite_size = Vec2::splat(128.0 / 2.0);
    let enemy = EnemyBundle {
        enemy: EnemyEntity,
        name: Name::new("Enemy Bundle"),
        sprite: Sprite {
            image,
            custom_size: Some(sprite_size),
            ..default()
        },
        transform: Transform::from_xyz(0.0, game_config.screen_height / 2.0 + -sprite_size.y, 0.1)
            .with_scale(Vec3::splat(1.0)),
        enemy_health: EnemyHealth(100),
        stats: Stats {
            strength: 10,
            agility: 10,
            stamina: 10,
            perception: 10,
            intelligence: 10,
        },
        effects: Effects {
            effects: Vec::new(),
        },
    };

    enemy
}

fn update_enemy_health(
    mut enemy_query: Query<(&mut EnemyHealth)>,
    mut enemy_health_text_query: Query<(&Parent, &mut Text2d), With<EnemyHealthText>>,
) {
    for (parent, mut health_text) in enemy_health_text_query.iter_mut() {
        let Ok(mut health) = enemy_query.get_mut(parent.get()) else {
            println!("No health text found");
            return;
        };
        health_text.0 = format!("{}", health.0);
    }
}
fn update_player_health(
    mut player_query: Query<(&mut PlayerHealth)>,
    mut player_health_text_query: Query<(&Parent, &mut Text2d), With<PlayerHealthText>>,
) {
    for (parent, mut health_text) in player_health_text_query.iter_mut() {
        let Ok(mut health) = player_query.get_mut(parent.get()) else {
            println!("No health text found");
            return;
        };
        health_text.0 = format!("{}", health.0);
    }
}

#[derive(Component, Clone)]
struct SelectableCard(bool);

#[derive(Component, Clone)]
struct PlayerCard;

#[derive(Component, Clone)]
struct EnemyCard;

#[derive(Component, Clone)]

struct Card {
    sprite: Sprite,
    selectable_card: SelectableCard,
    id: usize,
    description: String,
    effect: CardEffect,
    cooldown: f32,
}

#[derive(Component)]
struct DeckPile;

#[derive(Component)]
struct BattleEntity;

#[derive(Component)]
struct EnemyHealthText;
#[derive(Component)]
struct PlayerHealthText;

/// Set up a scene that tests all sprite anchor types.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, game_config: Res<GameConfig>) {
    println!("Setting up scene");
    commands.spawn(Camera2d);

    let len = 128.0;
    let sprite_size = Vec2::splat(len / 2.0);

    // let cards = [
    //     Card {
    //         sprite: Sprite {
    //             image: asset_server.load("boss_bee.png"),
    //             custom_size: Some(sprite_size),
    //             ..default()
    //         },
    //         selectable_card: SelectableCard(false),
    //         id: 1,
    //         description: "This is test card #1".to_string(),
    //         effect: CardEffect::DirectDamage(10),
    //         cooldown: 2.0,
    //     },
    //     Card {
    //         sprite: Sprite {
    //             image: asset_server.load("boss_bee.png"),
    //             custom_size: Some(sprite_size),
    //             ..default()
    //         },
    //         selectable_card: SelectableCard(false),
    //         id: 2,
    //         description: "test card #2".to_string(),
    //         effect: CardEffect::DamageOverTime {
    //             damage: 5,
    //             duration: 3.0,
    //             frequency: 0.5,
    //         },
    //         cooldown: 4.0,
    //     },
    // ];

    let screen_width = game_config.screen_width;
    let screen_height = game_config.screen_height;
    println!("Game config: {:?}", game_config);

    let num_cards = random_range(1..4);
    let starting_x = 0.0 - (num_cards / 2) as f32 * (sprite_size.x * 0.7);

    let mut current_x = starting_x;
    let playerbundle = spawn_player(asset_server.load("player.png"), sprite_size, screen_height);

    let playerid = commands.spawn(playerbundle).with_children(|parent| {
        // add health text
        parent.spawn((
            Name::new("Health Text"),
            Text2d::new("100"),
            Transform::from_xyz(0.0, -sprite_size.y, 0.1),
            PlayerHealthText,
        ));
        // add skills

        for i in 0..num_cards {
            let transform = get_card_transform((i, num_cards), sprite_size, CardPosition::Top);
            let cardeffect = CardEffect::get_random_effect();
            let sprite = cardeffect.get_sprite_path();
            let sprite_handle = asset_server.load(sprite);
            parent
                .spawn((
                    Name::new(format!("Player Card {}", i)),
                    Sprite {
                        image: sprite_handle,
                        custom_size: Some(sprite_size),
                        ..default()
                    },
                    SelectableCard(true),
                    transform,
                    PlayerCard,
                    cardeffect,
                    CardAttackTimer(Timer::from_seconds(
                        random_range(1.0..3.0),
                        TimerMode::Repeating,
                    )),
                ))
                // .observe(select_card_on::<Pointer<Click>>())
                // .observe(hover_card_on::<Pointer<Over>>())
                // .observe(hover_card_out::<Pointer<Out>>())
                .with_children(|parent| {
                    add_timer_bar(parent);
                });
            current_x += sprite_size.x * 0.7;
        }
    });

    // Spawn inventory button
    commands
        .spawn((
            Name::new("Inventory Button"),
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.3),
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            Transform::from_xyz(
                -game_config.screen_width / 2.0 + 30.0,
                game_config.screen_height / 2.0 - 30.0,
                0.9,
            ),
            InventoryButton,
        ))
        .with_children(|parent| {
            parent.spawn((Text2d::new("I"), Transform::from_xyz(0.0, 0.0, 0.1)));
        })
        .observe(change_sprite_color::<Pointer<Out>>(Color::srgb(
            0.0, 0.7, 0.5,
        )));
}

fn get_card_transform(
    num_cards: (i32, i32),
    sprite_size: Vec2,
    position: CardPosition,
) -> Transform {
    let (index, total) = num_cards;
    let card_spacing = sprite_size.x * 1.0; // Space between cards
    let total_width = card_spacing * (total - 1) as f32;
    let starting_x = -total_width / 2.0;
    let current_x = starting_x + (index as f32 * card_spacing);

    match position {
        CardPosition::Top => {
            Transform::from_xyz(current_x, sprite_size.y, 0.0).with_scale(Vec3::splat(1.0))
        }
        CardPosition::Bottom => {
            Transform::from_xyz(current_x, -sprite_size.y, 0.0).with_scale(Vec3::splat(1.0))
        }
    }
}

fn add_card(
    parent: &mut ChildBuilder,
    // card_image: Handle<Image>,
    asset_server: &Res<AssetServer>,
    sprite_size: Vec2,
    owner: impl Component,
    num_cards: (i32, i32),
    position: CardPosition,
) {
    let transform = get_card_transform(num_cards, sprite_size, position);
    let cardeffect = CardEffect::get_random_effect();
    let sprite = match cardeffect {
        CardEffect::DirectDamage(_) => asset_server.load("direct.png"),
        CardEffect::DamageOverTime { .. } => asset_server.load("dot.png"),
        CardEffect::Stun { .. } => asset_server.load("stun.png"),
        CardEffect::Heal(_) => asset_server.load("heal.png"),
    };

    parent
        .spawn((
            Name::new("Card"),
            Sprite {
                image: sprite,
                custom_size: Some(sprite_size),
                ..default()
            },
            transform,
            owner,
            cardeffect,
            CardAttackTimer(Timer::from_seconds(3.0, TimerMode::Repeating)),
            BattleEntity,
        ))
        .with_children(|parent| {
            add_timer_bar(parent);
        });
}

fn add_timer_bar(parent: &mut ChildBuilder) {
    parent.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(50.0, 5.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -20.0, 0.1),
    ));
    parent.spawn((
        Sprite {
            color: RED,
            custom_size: Some(Vec2::new(0.0, 5.0)), // Start at width 0
            ..default()
        },
        Transform::from_xyz(-25.0, -20.0, 0.2),
        CardTimerBar,
    ));
}

#[derive(Bundle)]
struct PlayerBundle {
    player: PlayerEntity,
    name: Name,
    sprite: Sprite,
    transform: Transform,
    player_health: PlayerHealth,
    stats: Stats,
    class: Class,
    effects: Effects,
}

fn spawn_player(image: Handle<Image>, sprite_size: Vec2, screen_height: f32) -> PlayerBundle {
    println!("Spawning player");
    let player = PlayerBundle {
        player: PlayerEntity,
        name: Name::new("Player"),
        sprite: Sprite {
            image,
            custom_size: Some(sprite_size),
            ..default()
        },
        transform: Transform::from_xyz(0.0, -screen_height / 2.0 + sprite_size.y, 0.1)
            .with_scale(Vec3::splat(1.0)),
        player_health: PlayerHealth(100),
        stats: Stats {
            strength: 20,
            agility: 10,
            stamina: 10,
            perception: 10,
            intelligence: 10,
        },
        class: Class::Warrior,
        effects: Effects {
            effects: Vec::new(),
        },
    };

    player
}

fn change_sprite_color<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(Trigger<E>, (Query<&mut Sprite>), Commands) {
    move |ev, (mut sprites), mut commands| {
        let Ok((mut sprite)) = sprites.get_mut(ev.entity()) else {
            return;
        };
        sprite.color = color;
    }
}

fn calculate_enemy_damage(enemy_stats: &Stats, player_stats: &Stats) -> usize {
    let enemy_strength = enemy_stats.strength;
    let player_agility = player_stats.agility;

    let dodge_chance: f32 = (player_stats.agility as f32 / enemy_stats.agility as f32) * 0.1;
    println!("Enemy dodge chance: {}", dodge_chance);
    if rand::random::<f32>() < dodge_chance {
        println!("Player dodged the attack!");
        return 0;
    }

    let base_damage = 5; // Base damage for enemy attacks
    let total_damage = base_damage * enemy_strength / player_agility;
    total_damage
}

fn calculate_player_damage(player_stats: &Stats, enemy_stats: &Stats) -> usize {
    let player_strength = player_stats.strength;
    let enemy_agility = enemy_stats.agility;

    let dodge_chance: f32 = (enemy_stats.agility as f32 / player_stats.agility as f32) * 0.1;
    println!("Player dodge chance: {}", dodge_chance);
    if rand::random::<f32>() < dodge_chance {
        println!("Enemy dodged the attack!");
        return 0;
    }

    let base_damage = 5; // Base damage for player attacks
    let total_damage = base_damage * player_strength / enemy_agility;
    total_damage
}

#[derive(Component)]
struct CardTimerBar;

fn calculate_player_effects(
    time: Res<Time>,
    mut player_effect_query: Query<(&mut Effects, &mut PlayerHealth), With<PlayerEntity>>,
) {
    // tick each of the effect timers
    let mut continued_effects: Vec<ActiveEffect> = vec![];
    let Ok((mut effects, mut player_health)) = player_effect_query.get_single_mut() else {
        println!("[calculate_effect_damage] No effects or player health found");
        return;
    };

    for effect in effects.effects.iter_mut() {
        match effect {
            ActiveEffect::DamageOverTime {
                damage,
                duration,
                frequency,
            } => {
                duration.tick(time.delta());
                frequency.tick(time.delta());
                if frequency.finished() {
                    player_health.0 -= *damage;
                    // frequency.reset();
                }
                if !duration.finished() {
                    continued_effects.push(effect.clone());
                }
            }
            ActiveEffect::DirectDamage(damage) => {
                player_health.0 -= *damage;
            }
            ActiveEffect::Stun { duration } => {
                duration.tick(time.delta());
                if !duration.finished() {
                    continued_effects.push(effect.clone());
                }
            }
            ActiveEffect::Heal(heal) => {
                player_health.0 += *heal;
            }
        }
    }
    effects.effects = continued_effects;
}
fn calculate_enemy_effects(
    time: Res<Time>,
    mut enemy_effect_query: Query<(&mut Effects, &mut EnemyHealth), With<EnemyEntity>>,
) {
    // tick each of the effect timers
    let mut continued_effects: Vec<ActiveEffect> = vec![];
    let Ok((mut effects, mut enemy_health)) = enemy_effect_query.get_single_mut() else {
        println!("[calculate_enemy_effects] No effects or enemy health found");
        return;
    };

    for effect in effects.effects.iter_mut() {
        match effect {
            ActiveEffect::DamageOverTime {
                damage,
                duration,
                frequency,
            } => {
                duration.tick(time.delta());
                frequency.tick(time.delta());
                if frequency.finished() {
                    enemy_health.0 -= *damage;
                    // frequency.reset();
                }
                if !duration.finished() {
                    continued_effects.push(effect.clone());
                }
            }
            ActiveEffect::DirectDamage(damage) => {
                enemy_health.0 -= *damage;
            }
            ActiveEffect::Stun { duration } => {
                duration.tick(time.delta());
                if !duration.finished() {
                    continued_effects.push(effect.clone());
                }
            }
            ActiveEffect::Heal(heal) => {
                enemy_health.0 += *heal;
            }
        }
    }
    effects.effects = continued_effects;
}

fn enemy_auto_attack(
    time: Res<Time>,
    mut enemy_cards_query: Query<(&mut CardAttackTimer, &CardEffect), With<EnemyCard>>,
    mut player_query: Query<(&mut Effects), With<PlayerEntity>>,
) {
    for (mut timer, effect) in enemy_cards_query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            println!("attack ready");
            let Ok((mut effects)) = player_query.get_single_mut() else {
                println!("[enemy_auto_attack] No player found");
                return;
            };

            match effect {
                CardEffect::DamageOverTime {
                    damage,
                    duration,
                    frequency,
                } => {
                    effects.effects.push(ActiveEffect::DamageOverTime {
                        damage: *damage,
                        duration: Timer::from_seconds(*duration, TimerMode::Repeating),
                        frequency: Timer::from_seconds(*frequency, TimerMode::Repeating),
                    });
                }
                CardEffect::DirectDamage(damage) => {
                    effects.effects.push(ActiveEffect::DirectDamage(*damage));
                }
                CardEffect::Stun { duration } => {
                    effects.effects.push(ActiveEffect::Stun {
                        duration: Timer::from_seconds(*duration, TimerMode::Repeating),
                    });
                }
                CardEffect::Heal(heal) => {
                    effects.effects.push(ActiveEffect::Heal(*heal));
                }
            }
        }
    }
}

fn player_auto_attack(
    time: Res<Time>,
    mut player_cards_query: Query<(&mut CardAttackTimer, &CardEffect), With<PlayerCard>>,
    mut enemy_query: Query<(&mut Effects), With<EnemyEntity>>,
) {
    for (mut timer, effect) in player_cards_query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            // println!("attack ready");
            let Ok((mut effects)) = enemy_query.get_single_mut() else {
                println!("[player_auto_attack] No enemy found");
                return;
            };

            match effect {
                CardEffect::DamageOverTime {
                    damage,
                    duration,
                    frequency,
                } => {
                    effects.effects.push(ActiveEffect::DamageOverTime {
                        damage: *damage,
                        duration: Timer::from_seconds(*duration, TimerMode::Repeating),
                        frequency: Timer::from_seconds(*frequency, TimerMode::Repeating),
                    });
                }
                CardEffect::DirectDamage(damage) => {
                    effects.effects.push(ActiveEffect::DirectDamage(*damage));
                }
                CardEffect::Stun { duration } => {
                    effects.effects.push(ActiveEffect::Stun {
                        duration: Timer::from_seconds(*duration, TimerMode::Repeating),
                    });
                }
                CardEffect::Heal(heal) => {
                    effects.effects.push(ActiveEffect::Heal(*heal));
                }
            }
        }
    }
}

fn check_enemy_death(
    enemy_query: Query<(Entity, &EnemyHealth)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    let mut alive_enemies = 0;
    for (entity, enemy_health) in enemy_query.iter() {
        match enemy_health.0.cmp(&0) {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                commands.entity(entity).despawn_recursive();
            }
            std::cmp::Ordering::Greater => {
                alive_enemies += 1;
            }
        }
    }
    if alive_enemies == 0 {
        next_state.set(GameState::LootScreen);
    }
}

#[derive(Component)]
struct LootScreen;

fn despawn_loot_screen(mut commands: Commands, loot_screen_query: Query<Entity, With<LootScreen>>) {
    if let Ok(loot_screen) = loot_screen_query.get_single() {
        commands.entity(loot_screen).despawn_recursive();
    }
}

fn spawn_loot_screen(mut commands: Commands, game_config: Res<GameConfig>) {
    let loot_items = (0..10)
        .map(|i| {
            let rand_rarity = random_range(0..3);
            let rarity = match rand_rarity {
                0 => LootRarity::Common,
                1 => LootRarity::Rare,
                _ => LootRarity::Epic,
            };
            let rand_item = random_range(0..3);
            let item = match rand_item {
                0 => "Health Potion",
                1 => "Magic Sword",
                _ => "Ancient Relic",
            };
            LootItem {
                name: format!("{}", item),
                rarity,
            }
        })
        .collect::<Vec<_>>();

    // Spawn background overlay
    let parent = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.2, 0.1, 0.0),
                custom_size: Some(Vec2::new(
                    game_config.screen_width,
                    game_config.screen_height,
                )),
                ..default()
            },
            LootScreen,
            Transform::from_xyz(0.0, 0.0, 0.9),
        ))
        .id();

    // Spawn loot items
    for (i, loot_item) in loot_items.iter().enumerate() {
        let y_pos = game_config.screen_height / 4.0 - (i as f32 + 1.0) * 50.0;

        commands.entity(parent).with_children(|parent| {
            parent
                .spawn((
                    Sprite {
                        color: loot_item.rarity.get_color(),
                        custom_size: Some(Vec2::new(200.0, 40.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, y_pos, 1.0),
                    loot_item.clone(),
                    Interaction::None,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text2d::new(&loot_item.name),
                        TextColor(loot_item.rarity.get_text_color()),
                        Transform::from_xyz(0.0, 0.0, 0.1),
                    ));
                });

            parent
                .spawn((
                    Sprite {
                        color: Color::rgb(0.3, 0.7, 0.3),
                        custom_size: Some(Vec2::new(120.0, 40.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -game_config.screen_height / 3.0, 1.0),
                    LootAllButton,
                    // Interaction::None,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text2d::new("Loot All"),
                        TextColor(Color::WHITE),
                        Transform::from_xyz(0.0, 0.0, 0.1),
                    ));
                })
                .observe(handle_loot_all::<Pointer<Click>>());
        });
    }
}

#[derive(Component)]
enum GameMenu {
    Inventory,
    Loot,
}

fn toggle_ui(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if input.just_pressed(KeyCode::KeyI) {
        if current_state.get() == &GameState::Battle {
            next_state.set(GameState::Menu);
        } else {
            next_state.set(GameState::Battle);
        }
    }
}

#[derive(Component)]
struct InventoryScroll {
    offset: f32,
    max_offset: f32,
}

fn spawn_menu(mut commands: Commands, resource: Res<Inventory>, game_config: Res<GameConfig>) {
    let total_items = resource.items.len() as f32;
    let max_scroll =
        (total_items * INVENTORY_ITEM_HEIGHT) - (INVENTORY_VISIBLE_ITEMS * INVENTORY_ITEM_HEIGHT);
    let max_scroll = max_scroll.max(0.0); // Don't allow negative scroll range

    // Background panel
    commands
        .spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.9),
                custom_size: Some(Vec2::new(
                    game_config.screen_width * 0.4,
                    INVENTORY_VISIBLE_ITEMS * INVENTORY_ITEM_HEIGHT, // Fixed height based on visible items
                )),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.9),
            InventoryDisplay,
            InventoryScroll {
                offset: 0.0,
                max_offset: max_scroll,
            },
            Visibility::default(),
            GameMenu::Inventory,
        ))
        .with_children(|parent| {
            // Title (above the scroll area)
            parent.spawn((
                Text2d::new("Inventory"),
                TextColor(Color::WHITE),
                Transform::from_xyz(
                    0.0,
                    INVENTORY_VISIBLE_ITEMS * INVENTORY_ITEM_HEIGHT / 2.0 + 20.0,
                    0.1,
                ),
            ));

            // Items container
            for (i, item) in resource.items.iter().enumerate() {
                let y_offset = -((i as f32) * INVENTORY_ITEM_HEIGHT);

                parent
                    .spawn((
                        Sprite {
                            color: item.rarity.get_color(),
                            custom_size: Some(Vec2::new(
                                game_config.screen_width * 0.35,
                                INVENTORY_ITEM_HEIGHT - 10.0, // Leave some spacing
                            )),
                            ..default()
                        },
                        Transform::from_xyz(0.0, y_offset, 0.1),
                    ))
                    .with_children(|item_parent| {
                        item_parent.spawn((
                            Text2d::new(&item.name),
                            TextColor(item.rarity.get_text_color()),
                            Transform::from_xyz(-game_config.screen_width * 0.15, 0.0, 0.1),
                        ));
                    });
            }
        });
}

fn despawn_menu(mut commands: Commands, menu_query: Query<Entity, With<GameMenu>>) {
    if let Ok(menu) = menu_query.get_single() {
        commands.entity(menu).despawn_recursive();
    }
}

fn handle_loot_all<E: Debug + Clone + Reflect>() -> impl Fn(
    Trigger<E>,
    (
        Query<&LootItem>,
        ResMut<NextState<GameState>>,
        ResMut<Inventory>,
        Commands,
    ),
) {
    move |ev, (loot_query, mut next_state, mut inventory, mut commands)| {
        println!("handle_loot_all");
        for loot_item in loot_query.iter() {
            println!("loot_item: {}", loot_item.name);
            inventory.items.push(loot_item.clone());
        }

        next_state.set(GameState::Battle);
    }
}

fn update_skill_timer_bars(
    card_query: Query<(&CardAttackTimer, &Children)>,
    mut timer_bar_query: Query<(&mut Transform, &mut Sprite), With<CardTimerBar>>,
) {
    for (attack_timer, children) in card_query.iter() {
        for child in children.iter() {
            if let Ok((mut transform, mut sprite)) = timer_bar_query.get_mut(*child) {
                let progress = attack_timer.elapsed_secs() / attack_timer.duration().as_secs_f32();
                let bar_width = 50.0;

                sprite.custom_size = Some(Vec2::new(bar_width * progress, 5.0));
                transform.translation.x = -25.0 + (bar_width * progress / 2.0);

                sprite.color = if progress < 0.3 {
                    RED
                } else if progress < 0.6 {
                    YELLOW
                } else {
                    GREEN
                };
            }
        }
    }
}

fn update_card_timers(mut card_query: Query<&mut CardAttackTimer>, time: Res<Time>) {
    for mut attack_timer in card_query.iter_mut() {
        attack_timer.0.tick(time.delta());
    }
}

fn despawn_battle_entities(
    mut commands: Commands,
    battle_entity_query: Query<Entity, With<BattleEntity>>,
    enemy_entity_query: Query<Entity, With<EnemyEntity>>,
) {
    println!("Despawning battle entities");
    for entity in battle_entity_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in enemy_entity_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_inventory_scroll(
    mut scroll_query: Query<(&mut InventoryScroll, &Children)>,
    mut item_query: Query<&mut Transform>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut scroll, children) in scroll_query.iter_mut() {
        let mut scroll_direction = 0.0;

        if input.pressed(KeyCode::ArrowUp) {
            scroll_direction += 1.0;
        }
        if input.pressed(KeyCode::ArrowDown) {
            scroll_direction -= 1.0;
        }

        // println!("scroll_direction: {}", scroll_direction);
        if scroll_direction != 0.0 {
            // Update scroll offset
            scroll.offset += scroll_direction * SCROLL_SPEED * time.delta().as_secs_f32();
            // println!("scroll.offset: {}", scroll.offset);
            scroll.offset = scroll.offset.clamp(0.0, scroll.max_offset);

            // Update item positions
            for child in children.iter() {
                if let Ok(mut transform) = item_query.get_mut(*child) {
                    let original_y = transform.translation.y;
                    if scroll_direction > 0.0 {
                        transform.translation.y = original_y + scroll.offset;
                    } else {
                        transform.translation.y = original_y - scroll.offset;
                    }
                }
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bee Game".to_string(),
                        resolution: WindowResolution::new(640.0, 480.0),
                        ..default()
                    }),
                    ..default()
                }),
            WorldInspectorPlugin::new(),
        ))
        // .add_plugins(InspectorPlugin::<GameState>::new())
        // .add_plugins(InspectorPlugin::<GameConfig>::new())
        // .add_plugins(WorldInspectorPlugin::new())
        .init_state::<GameState>()
        .init_resource::<Inventory>()
        .add_systems(Startup, (setup))
        .add_systems(
            Update,
            (
                update_enemy_health,
                update_player_health,
                update_card_timers,
                update_skill_timer_bars,
                enemy_auto_attack,
                player_auto_attack,
                calculate_player_effects,
                calculate_enemy_effects,
                // enemy_skill_auto_attack,
                check_enemy_death,
                // handle_inventory_button,
            )
                .chain()
                .run_if(in_state(GameState::Battle)),
        )
        .add_systems(Update, (debug_display_state, toggle_ui))
        .add_systems(OnEnter(GameState::Battle), on_enter_battle)
        .add_systems(
            OnEnter(GameState::LootScreen),
            (despawn_battle_entities, spawn_loot_screen).chain(),
        )
        // .add_systems(OnEnter(GameState::EndBattle), despawn_battle_entities)
        .add_systems(OnExit(GameState::LootScreen), despawn_loot_screen)
        .add_systems(OnEnter(GameState::Menu), spawn_menu)
        .add_systems(OnExit(GameState::Menu), despawn_menu)
        .add_systems(
            Update,
            handle_inventory_scroll.run_if(in_state(GameState::Menu)),
        )
        .insert_resource(GameConfig {
            screen_width: 640.0,
            screen_height: 480.0,
        })
        .register_type::<Effects>()
        .run();
}
