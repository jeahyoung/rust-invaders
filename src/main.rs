use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod components;
mod enemy;
mod player;

use crate::components::{
    Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, FromPlayer, Laser, Movable, SpriteSize,
    Velocity,
};
use enemy::EnemyPlugin;
use player::PlayerPlugin;

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_SIZE: (f32, f32) = (144., 75.);
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);

const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (17., 55.);

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LEN: usize = 16;

const SPRITE_SCALE: (f32, f32) = (0.5, 0.5);

const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 50.;

const ENEMY_MAX: u32 = 2;

#[derive(Resource, Debug)]
pub struct WinSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct GameTexture {
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    explosion: Handle<TextureAtlas>,
}

#[derive(Resource)]
struct EnemyCount {
    count: u32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Invaders!".into(),
                resolution: (500., 700.).into(),
                ..default()
            }),
            ..default()
        }))
        // Inspector Setup
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_systems((setup_camera, setup_system))
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_systems((
            movable_system,
            player_laser_hit_enemy_system,
            explosion_to_spawn_system,
            explosion_animation_system,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_system(
    mut commands: Commands,
    query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let Ok(primary) = query.get_single() else { return; };
    let (win_width, win_height) = (primary.width(), primary.height());

    let win_size = WinSize {
        width: win_width,
        height: win_height,
    };

    commands.insert_resource(win_size);

    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4, None, None);
    let explosion = texture_atlases.add(texture_atlas);

    let game_texture = GameTexture {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
        explosion,
    };

    commands.insert_resource(game_texture);
    commands.insert_resource(EnemyCount { count: 0 });
}

fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
    query.for_each_mut(|(entity, velocity, mut transform, moveable)| {
        transform.translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        transform.translation.y += velocity.y * TIME_STEP * BASE_SPEED;

        if moveable.auto_despawn {
            let margin: f32 = 200.;
            if transform.translation.x > win_size.width / 2. + margin
                || transform.translation.x < -win_size.width / 2. - margin
                || transform.translation.y > win_size.height / 2. + margin
                || transform.translation.y < -win_size.height / 2. - margin
            {
                // println!("->> despawned {entity:?}");
                // println!(
                //     "->> translation {:?} win_size {:?}",
                //     transform.translation, win_size
                // );
                commands.entity(entity).despawn();
            }
        }
    });
}

fn player_laser_hit_enemy_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
    laser_query.for_each(|(laser_entity, laser_transform, laser_sprite_size)| {
        let laser_scale = laser_transform.scale.xy();
        enemy_query.for_each(|(enemy_entity, enemy_transform, enemy_sprite_size)| {
            let enemy_scale = enemy_transform.scale.xy();
            // determine if collision
            let collision = collide(
                laser_transform.translation,
                laser_sprite_size.0 * laser_scale,
                enemy_transform.translation,
                enemy_sprite_size.0 * enemy_scale,
            );

            //perform collision
            if collision.is_some() {
                commands.entity(laser_entity).despawn();
                commands.entity(enemy_entity).despawn();
                commands.spawn(ExplosionToSpawn(enemy_transform.translation));
                println!("Despawn Enemy count: {:?}", enemy_count.count);
                enemy_count.count -= 1;
            }
        })
    })
}

fn explosion_to_spawn_system(
    mut commands: Commands,
    game_texture: Res<GameTexture>,
    query: Query<(Entity, &ExplosionToSpawn), With<ExplosionToSpawn>>,
) {
    query.for_each(|(explosion_spawn_entity, explosion_to_transform)| {
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: game_texture.explosion.clone(),
                transform: Transform::from_translation(explosion_to_transform.0),
                ..default()
            },
            Explosion,
            ExplosionTimer::default(),
        ));

        commands.entity(explosion_spawn_entity).despawn();
    })
}

fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
    query.for_each_mut(|(explosion_entity, mut explosion_timer, mut sprite)| {
        explosion_timer.0.tick(time.delta());
        if explosion_timer.0.finished() {
            sprite.index += 1;
            if sprite.index >= EXPLOSION_LEN {
                commands.entity(explosion_entity).despawn();
            }
        } else {
            //sprite.index = (sprite.index + 1) % EXPLOSION_LEN;
        }
    })
}
