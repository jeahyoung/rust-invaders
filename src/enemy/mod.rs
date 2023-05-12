use crate::components::{Enemy, FromEnemy, Laser, Movable, SpriteSize, Velocity};
use crate::enemy::formation::{Formation, FormationMaker};
use crate::{
    EnemyCount, GameTexture, WinSize, BASE_SPEED, ENEMY_LASER_SIZE, ENEMY_MAX, ENEMY_SIZE,
    SPRITE_SCALE, TIME_STEP,
};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use rand::{thread_rng, Rng};
use std::f32::consts::PI;
use std::time::Duration;

mod formation;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FormationMaker::default())
            //.add_startup_systems((enemy_spawn_system.in_base_set(StartupSet::PostStartup),))
            .add_systems((
                enemy_spawn_system.run_if(on_timer(Duration::from_secs(1))),
                enemy_fire_system.run_if(enemy_fire_criteria),
                enemy_movement_system,
            ));
    }
}

fn enemy_spawn_system(
    mut commands: Commands,
    game_texture: Res<GameTexture>,
    mut enemy_count: ResMut<EnemyCount>,
    mut formation_maker: ResMut<FormationMaker>,
    win_size: Res<WinSize>,
) {
    if enemy_count.count < ENEMY_MAX {
        /*let mut rng = thread_rng();
        let w_span = win_size.width / 2. - 100.;
        let h_span = win_size.height / 2. - 100.;
        let x = rng.gen_range(-w_span..w_span);
        let y = rng.gen_range(-h_span..h_span);*/

        // println!(
        //     "win_size.width: {}, win_size.height: {}",
        //     win_size.width, win_size.height
        // );
        // println!("w_span: {}, h_span: {}", w_span, h_span);
        // println!("x: {}, y: {}", x, y);
        let formation = formation_maker.make(&win_size);
        let (x, y) = formation.start;

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
            formation,
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
                    rotation: Quat::from_rotation_x(PI),
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

fn enemy_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Formation), With<Enemy>>,
) {
    let now = time.elapsed_seconds();
    for (mut transform, mut formation) in query.iter_mut() {
        let (x_org, y_org) = (transform.translation.x, transform.translation.y);
        let max_distance = TIME_STEP * formation.speed;

        let dir = if formation.start.0 < 0. { 1. } else { -1. };
        let (x_pivot, y_pivot) = formation.pivot;
        let (x_radius, y_radius) = formation.radius;

        let angle = formation.angle
            + dir * formation.speed * TIME_STEP / (x_radius.min(y_radius) * PI / 2.);

        let x_dst = x_radius * angle.cos() + x_pivot;
        let y_dst = y_radius * angle.sin() + y_pivot;

        let dx = x_org - x_dst;
        let dy = y_org - y_dst;
        let distance = (dx * dx + dy * dy).sqrt();
        let distance_ratio = if distance != 0. {
            max_distance / distance
        } else {
            0.
        };

        let x = x_org - dx * distance_ratio;
        let x = if dx > 0. { x.max(x_dst) } else { x.min(x_dst) };
        let y = y_org + dy * distance_ratio;
        let y = if dy > 0. { y.max(y_dst) } else { y.min(y_dst) };

        if distance < max_distance * formation.speed / 20. {
            formation.angle = angle;
        }

        (transform.translation.x, transform.translation.y) = (x, y);
    }
}
