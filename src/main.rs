//! Demonstrates picking for sprites and sprite atlases. The picking backend only tests against the
//! sprite bounds, so the sprite atlas can be picked by clicking on its transparent areas.

use bevy::{prelude::*, sprite::Anchor, window::WindowResolution};

use std::fmt::Debug;

#[derive(Resource, Debug)]
struct GameConfig {
    screen_width: f32,
    screen_height: f32,
}

fn main() {
    App::new()
        .add_plugins(
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
        )
        .add_systems(Startup, (setup, setup_atlas))
        .add_systems(Update, (move_sprite, animate_sprite, update_health))
        .insert_resource(GameConfig {
            screen_width: 640.0,
            screen_height: 480.0,
        })
        .run();
}

fn update_health(
    mut enemy_query: Query<(&mut EnemyHealth, &mut Text2d), Without<PlayerHealth>>,
    mut player_query: Query<(&mut PlayerHealth, &mut Text2d), Without<EnemyHealth>>,
) {
    for (mut enemy_health, mut enemy_text) in &mut enemy_query {
        enemy_text.0 = format!("Health: {}", enemy_health.0);
    }
    for (mut player_health, mut player_text) in &mut player_query {
        player_text.0 = format!("Health: {}", player_health.0);
    }
}

fn move_sprite(
    time: Res<Time>,
    mut sprite: Query<&mut Transform, (Without<Sprite>, With<Children>)>,
) {
    let t = time.elapsed_secs() * 0.1;
    for mut transform in &mut sprite {
        let new = Vec2 {
            x: 50.0 * ops::sin(t),
            y: 50.0 * ops::sin(t * 2.0),
        };
        transform.translation.x = new.x;
        transform.translation.y = new.y;
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
/// Set up a scene that tests all sprite anchor types.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, game_config: Res<GameConfig>) {
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

    let middle_x = screen_width / 2.0;
    // let starting_x = middle_x - ((cards.len() / 3) as f32 * sprite_size.x / 2.0);
    let starting_x = 0.0 - (cards.len() / 2) as f32 * sprite_size.x;
    let mut current_x = starting_x;

    for (i, card) in cards.iter().enumerate() {
        commands
            .spawn((
                card.sprite.clone(),
                card.selectable_card.clone(),
                Transform::from_xyz(current_x, -screen_height / 3.0, 0.0)
                    .with_scale(Vec3::splat(1.0)),
                Text2d::new(card.description.clone()),
                PlayerCard,
                Damage(10),
            ))
            .observe(select_card_on::<Pointer<Click>>(Color::srgb(0.3, 0.0, 1.0)));
        current_x += sprite_size.x;
    }

    commands.spawn((
        Card {
            sprite: Sprite {
                image: asset_server.load("boss_bee.png"),
                custom_size: Some(sprite_size),
                ..default()
            },
            selectable_card: SelectableCard(false),
            id: 3,
            description: "test card #3".to_string(),
        },
        EnemyCard,
    ));

    commands
        .spawn((
            Sprite {
                image: asset_server.load("boss_bee.png"),
                custom_size: Some(sprite_size),
                ..default()
            },
            Transform::from_xyz(0.0, screen_height / 2.0 + -sprite_size.y, 0.1)
                .with_scale(Vec3::splat(1.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                EnemyHealth(100),
                Text2d::new("Enemy"),
                Transform::from_xyz(0.0, -30.0, 0.1).with_scale(Vec3::splat(1.0)),
            ));
        });
    commands
        .spawn((
            Sprite {
                image: asset_server.load("player.png"),
                custom_size: Some(sprite_size),
                ..default()
            },
            Transform::from_xyz(0.0, -screen_height / 2.0 + sprite_size.y, 0.1)
                .with_scale(Vec3::splat(1.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerHealth(100),
                Text2d::new("Player"),
                Transform::from_xyz(0.0, -30.0, 0.1).with_scale(Vec3::splat(1.0)),
            ));
        });
}

#[derive(Component)]
struct EnemyHealth(usize);

#[derive(Component)]
struct PlayerHealth(usize);
#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        let Some(texture_atlas) = &mut sprite.texture_atlas else {
            continue;
        };

        timer.tick(time.delta());

        if timer.just_finished() {
            texture_atlas.index = if texture_atlas.index == indices.last {
                indices.first
            } else {
                texture_atlas.index + 1
            };
        }
    }
}

fn setup_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("gabe-idle-run.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(24, 24), 7, 1, None, None);
    let texture_atlas_layout_handle = texture_atlas_layouts.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };
    commands
        .spawn((
            Sprite::from_atlas_image(
                texture_handle,
                TextureAtlas {
                    layout: texture_atlas_layout_handle,
                    index: animation_indices.first,
                },
            ),
            Transform::from_xyz(300.0, 0.0, 0.0).with_scale(Vec3::splat(6.0)),
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            SelectableCard(true),
        ))
        .observe(recolor_on::<Pointer<Over>>(Color::srgb(0.0, 1.0, 1.0)))
        .observe(recolor_on::<Pointer<Out>>(Color::srgb(1.0, 1.0, 1.0)))
        .observe(recolor_on::<Pointer<Down>>(Color::srgb(1.0, 1.0, 0.0)))
        .observe(recolor_on::<Pointer<Up>>(Color::srgb(0.0, 1.0, 1.0)));
}

// An observer listener that changes the target entity's color.
fn recolor_on<E: Debug + Clone + Reflect>(color: Color) -> impl Fn(Trigger<E>, Query<&mut Sprite>) {
    move |ev, mut sprites| {
        let Ok(mut sprite) = sprites.get_mut(ev.entity()) else {
            return;
        };
        sprite.color = color;
    }
}

fn select_card_on<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(
    Trigger<E>,
    (
        Query<(&mut Sprite, &mut SelectableCard, &Damage)>,
        Query<&mut EnemyHealth>,
    ),
) {
    move |ev, (mut sprites, mut enemy)| {
        let Ok((mut sprite, mut selectable_card, damage)) = sprites.get_mut(ev.entity()) else {
            return;
        };
        selectable_card.0 = !selectable_card.0;
        sprite.color = if selectable_card.0 {
            color
        } else {
            Color::WHITE
        };
        for mut enemy in &mut enemy {
            enemy.0 -= damage.0;
        }
    }
}
