//! Demonstrates picking for sprites and sprite atlases. The picking backend only tests against the
//! sprite bounds, so the sprite atlas can be picked by clicking on its transparent areas.

use bevy::{
    prelude::*,
    sprite::{self, Anchor},
    state::commands,
    ui::Interaction,
    window::WindowResolution,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::random_range;
use skills::skills::{Class, Stats};
use std::fmt::Debug;

mod card;
mod skills;

#[derive(Resource, Debug, Default)]
struct GameConfig {
    screen_width: f32,
    screen_height: f32,
}

#[derive(Component)]
struct EnemyHealth(usize);

#[derive(Component)]
struct PlayerHealth(usize);

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
            LootRarity::Common => Color::WHITE,
            LootRarity::Rare => Color::srgb(0.0, 0.5, 1.0),
            LootRarity::Epic => Color::srgb(0.8, 0.0, 0.8),
        }
    }

    fn get_text_color(&self) -> Color {
        match self {
            LootRarity::Common => Color::BLACK,
            LootRarity::Rare => Color::WHITE,
            LootRarity::Epic => Color::WHITE,
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
}

fn on_enter_battle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_config: Res<GameConfig>,
) {
    let enemybundle = spawn_new_enemy(asset_server.load("boss_bee.png"), &game_config);
    let enemyid = commands.spawn(enemybundle).id();

    commands.entity(enemyid).with_children(|parent| {
        add_card(
            parent,
            asset_server.load("player.png"),
            Vec2::splat(128.0 / 2.0),
            EnemyCard,
        );
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
}

#[derive(Component, Clone)]
struct Damage(usize);

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

    let cards = [
        Card {
            sprite: Sprite {
                image: asset_server.load("boss_bee.png"),
                custom_size: Some(sprite_size),
                ..default()
            },
            selectable_card: SelectableCard(false),
            id: 1,
            description: "This is test card #1".to_string(),
        },
        Card {
            sprite: Sprite {
                image: asset_server.load("boss_bee.png"),
                custom_size: Some(sprite_size),
                ..default()
            },
            selectable_card: SelectableCard(false),
            id: 2,
            description: "test card #2".to_string(),
        },
    ];

    let screen_width = game_config.screen_width;
    let screen_height = game_config.screen_height;
    println!("Game config: {:?}", game_config);

    let starting_x = 0.0 - (cards.len() / 2) as f32 * (sprite_size.x * 0.7);
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

        for (i, card) in cards.iter().enumerate() {
            parent
                .spawn((
                    Name::new(format!("Player Card {}", i)),
                    card.sprite.clone(),
                    card.selectable_card.clone(),
                    Transform::from_xyz(current_x, sprite_size.y, 0.0).with_scale(Vec3::splat(1.0)),
                    PlayerCard,
                    Damage(10),
                    CardAttackTimer(Timer::from_seconds(
                        random_range(1.0..3.0),
                        TimerMode::Repeating,
                    )),
                    // BattleEntity,
                ))
                // .observe(select_card_on::<Pointer<Click>>())
                .observe(hover_card_on::<Pointer<Over>>())
                .observe(hover_card_out::<Pointer<Out>>())
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
        )))
        .observe(handle_inventory_button::<Pointer<Click>>());
}

fn add_card(
    parent: &mut ChildBuilder,
    card_image: Handle<Image>,
    sprite_size: Vec2,
    owner: impl Component,
) {
    parent
        .spawn((
            Name::new("Card"),
            Sprite {
                image: card_image,
                custom_size: Some(sprite_size),
                ..default()
            },
            Transform::from_xyz(0.0, -sprite_size.y, 0.0).with_scale(Vec3::splat(1.0)),
            owner,
            Damage(10),
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

fn hover_card_on<E: Debug + Clone + Reflect>(
) -> impl Fn(Trigger<E>, Query<(&mut Transform, &mut SelectableCard, &Damage)>) {
    move |ev, (mut sprites)| {
        let Ok((mut transform, mut selectable_card, damage)) = sprites.get_mut(ev.entity()) else {
            println!("No selectable card found");
            return;
        };
        transform.translation.y += 10.0;
    }
}

fn hover_card_out<E: Debug + Clone + Reflect>(
) -> impl Fn(Trigger<E>, Query<(&mut Transform, &mut SelectableCard, &Damage)>) {
    move |ev, (mut sprites)| {
        let Ok((mut transform, mut selectable_card, damage)) = sprites.get_mut(ev.entity()) else {
            println!("No selectable card found");
            return;
        };
        transform.translation.y -= 10.0;
    }
}

fn calculate_damage(player_stats: &Stats, enemy_stats: &Stats, damage: usize) -> usize {
    let player_strength = player_stats.strength;
    let enemy_agility = enemy_stats.agility;

    let dodge_chance: f32 = (player_stats.agility as f32 / enemy_stats.agility as f32) * 0.5;
    println!("Dodge chance: {}", dodge_chance);
    if rand::random::<f32>() < dodge_chance {
        println!("Dodge!");
        return 0;
    }

    let total_damage = damage * player_strength / enemy_agility;
    total_damage
}

fn calculate_enemy_damage(enemy_stats: &Stats, player_stats: &Stats) -> usize {
    let enemy_strength = enemy_stats.strength;
    let player_agility = player_stats.agility;

    let dodge_chance: f32 = (player_stats.agility as f32 / enemy_stats.agility as f32) * 0.5;
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

    let dodge_chance: f32 = (enemy_stats.agility as f32 / player_stats.agility as f32) * 0.5;
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

fn enemy_auto_attack(
    time: Res<Time>,
    mut enemy_cards_query: Query<(&mut CardAttackTimer), With<EnemyCard>>,
    mut player_query: Query<(&mut PlayerHealth, &Stats), With<PlayerEntity>>,
    mut enemy_query: Query<(&mut EnemyHealth, &Stats), With<EnemyEntity>>,
) {
    let Ok((mut player_health, player_stats)) = player_query.get_single_mut().map_err(|err| {
        println!("[enemy_auto_attack] No player found: {:?}", err);
        return;
    }) else {
        println!("[enemy_auto_attack] No player found");
        return;
    };
    let Ok((mut enemy_health, enemy_stats)) = enemy_query.get_single_mut().map_err(|err| {
        println!("[enemy_auto_attack] No enemy found: {:?}", err);
        return;
    }) else {
        println!("[enemy_auto_attack] No enemy found");
        return;
    };

    for (mut timer) in enemy_cards_query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            println!("attack ready");
            let damage = calculate_enemy_damage(&enemy_stats, &player_stats);
            player_health.0 = player_health.0.saturating_sub(damage);
            println!(
                "Enemy auto-attacks for {} damage! Player health: {}",
                damage, player_health.0
            );
        }
    }
}

fn player_auto_attack(
    time: Res<Time>,
    mut player_cards_query: Query<(&mut CardAttackTimer), With<PlayerCard>>,
    mut enemy_query: Query<(&mut EnemyHealth, &Stats), With<EnemyEntity>>,
    mut player_stats_query: Query<&Stats, With<PlayerHealth>>,
) {
    let Ok((mut enemy_health, enemy_stats)) = enemy_query.get_single_mut() else {
        println!("[player_auto_attack] No enemy found");
        return;
    };
    let Ok(player_stats) = player_stats_query.get_single() else {
        println!("[player_auto_attack] No player stats found");
        return;
    };

    for (mut timer) in player_cards_query.iter_mut() {
        timer.0.tick(time.delta());
        // println!("player_auto_attack: timer: {:?}", timer.0.elapsed_secs());
        if timer.0.finished() {
            let damage = calculate_player_damage(&player_stats, &enemy_stats);
            enemy_health.0 = enemy_health.0.saturating_sub(damage);
            println!(
                "Player auto-attacks for {} damage! Enemy health: {}",
                damage, enemy_health.0
            );
        }
    }
}

fn check_enemy_death(
    enemy_query: Query<(Entity, &EnemyHealth)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if let Ok((entity, enemy_health)) = enemy_query.get_single() {
        if enemy_health.0 == 0 {
            next_state.set(GameState::LootScreen);
            commands.entity(entity).despawn_recursive();
        }
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
    let loot_items = vec![
        LootItem {
            name: "Health Potion".to_string(),
            rarity: LootRarity::Common,
        },
        LootItem {
            name: "Magic Sword".to_string(),
            rarity: LootRarity::Rare,
        },
        LootItem {
            name: "Ancient Relic".to_string(),
            rarity: LootRarity::Epic,
        },
    ];

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

fn handle_inventory_button<E: Debug + Clone + Reflect>() -> impl Fn(
    Trigger<E>,
    (
        Query<(&Transform), With<InventoryDisplay>>,
        Res<Inventory>,
        Commands,
    ),
) {
    println!("handle_inventory_button triggered");
    // display inventory
    move |ev, (inventory_display_query, inventory, mut commands)| {
        let Ok(transform) = inventory_display_query.get_single() else {
            println!("[handle_inventory_button] No inventory display found");
            return;
        };
        spawn_inventory_display(&mut commands, &transform, &inventory);
    }
}

fn spawn_inventory_display(
    commands: &mut Commands,
    button_transform: &Transform,
    inventory: &Inventory,
) {
    let display_entity = commands
        .spawn((
            Sprite {
                color: Color::rgba(0.0, 0.0, 0.0, 0.8),
                custom_size: Some(Vec2::new(200.0, inventory.items.len() as f32 * 30.0 + 20.0)),
                ..default()
            },
            Transform::from_xyz(
                button_transform.translation.x + 120.0,
                button_transform.translation.y - 20.0,
                0.95,
            ),
            InventoryDisplay,
        ))
        .id();

    // Spawn items in inventory
    for (i, item) in inventory.items.iter().enumerate() {
        commands.entity(display_entity).with_children(|parent| {
            parent
                .spawn((
                    Sprite {
                        color: item.rarity.get_color(),
                        custom_size: Some(Vec2::new(180.0, 25.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -(i as f32 * 30.0 + 10.0), 0.1),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text2d::new(&item.name),
                        TextColor(item.rarity.get_text_color()),
                        Transform::from_xyz(0.0, 0.0, 0.1),
                    ));
                });
        });
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
                update_skill_timer_bars,
                enemy_auto_attack,
                player_auto_attack,
                // enemy_skill_auto_attack,
                check_enemy_death,
                // handle_inventory_button,
            )
                .chain()
                .run_if(in_state(GameState::Battle)),
        )
        .add_systems(Update, debug_display_state)
        .add_systems(OnEnter(GameState::Battle), on_enter_battle)
        .add_systems(OnEnter(GameState::LootScreen), spawn_loot_screen)
        .add_systems(OnExit(GameState::Battle), despawn_battle_entities)
        .add_systems(OnExit(GameState::LootScreen), despawn_loot_screen)
        .insert_resource(GameConfig {
            screen_width: 640.0,
            screen_height: 480.0,
        })
        .run();
}
