use crate::components::{Enemy, FromEnemy, Laser, Movable, SpriteSize, Velocity};
use crate::{
    EnemyCount, GameTexture, WinSize, ENEMY_LASER_SIZE, ENEMY_MAX, ENEMY_SIZE, SPRITE_SCALE,
};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use rand::{thread_rng, Rng};
use std::time::Duration;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_startup_systems((enemy_spawn_system.in_base_set(StartupSet::PostStartup),))
            .add_systems((
                enemy_spawn_system.run_if(on_timer(Duration::from_secs(1))),
                enemy_fire_system.run_if(enemy_fire_criteria),
            ));
    }
}

fn enemy_spawn_system(
    mut commands: Commands,
    game_texture: Res<GameTexture>,
    mut enemy_count: ResMut<EnemyCount>,
    win_size: Res<WinSize>,
) {
    if enemy_count.count < ENEMY_MAX {
        let mut rng = thread_rng();
        let w_span = win_size.width / 2. - 100.;
        let h_span = win_size.height / 2. - 100.;
        let x = rng.gen_range(-w_span..w_span);
        let y = rng.gen_range(-h_span..h_span);

        // println!(
        //     "win_size.width: {}, win_size.height: {}",
        //     win_size.width, win_size.height
        // );
        // println!("w_span: {}, h_span: {}", w_span, h_span);
        // println!("x: {}, y: {}", x, y);
        commands.spawn((
            SpriteBundle {
                texture: game_texture.enemy.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y, 10.),
                    scale: Vec3::new(SPRITE_SCALE.0, SPRITE_SCALE.1, 1.0),
                    ..default()
                },
                ..default()
            },
            Enemy,
            SpriteSize::from(ENEMY_SIZE),
            Name::new("Enemy"),
        ));
        enemy_count.count += 1;
        println!("Created Enemy count: {:?}", enemy_count.count);
    }
}

fn enemy_fire_criteria() -> bool {
    thread_rng().gen_bool(1. / 60.0)
}

fn enemy_fire_system(
    mut commands: Commands,
    game_texture: Res<GameTexture>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    enemy_query.for_each(|transform| {
        let (x, y) = (transform.translation.x, transform.translation.y);
        commands.spawn((
            SpriteBundle {
                texture: game_texture.enemy_laser.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y - 15.0, 0.),
                    scale: Vec3::new(SPRITE_SCALE.0, SPRITE_SCALE.1, 1.0),
                    ..default()
                },
                ..default()
            },
            Laser,
            SpriteSize::from(ENEMY_LASER_SIZE),
            FromEnemy,
            Movable { auto_despawn: true },
            Velocity { x: 0., y: -1. },
            Name::new("Laser"),
        ));
    })
}
