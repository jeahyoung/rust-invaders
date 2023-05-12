use crate::components::{FromPlayer, Laser, Movable, Player, SpriteSize, Velocity};
use crate::{
    GameTexture, PlayerState, WinSize, BASE_SPEED, PLAYER_LASER_SIZE, PLAYER_RESPAWN_DELAY,
    PLAYER_SIZE, SPRITE_SCALE, TIME_STEP,
};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerState::default())
            //.add_startup_systems((player_spawn_system.in_base_set(StartupSet::PostStartup),))
            .add_systems((
                //player_movement_system,
                player_spawn_system.run_if(on_timer(Duration::from_secs_f32(0.5))),
                player_keyboard_event_system,
                player_fire_system,
            ));
    }
}

fn player_spawn_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    game_texture: Res<GameTexture>,
    win_size: Res<WinSize>,
) {
    let now = time.elapsed_seconds_f64();
    let last_shot = player_state.last_shot;

    if !player_state.on && (last_shot == -1. || now > last_shot + PLAYER_RESPAWN_DELAY) {
        let bottom = -win_size.height / 2.0;
        commands.spawn((
            SpriteBundle {
                texture: game_texture.player.clone(),
                transform: Transform {
                    translation: Vec3::new(
                        0.,
                        bottom + PLAYER_SIZE.1 / 2. * SPRITE_SCALE.1 + 5.,
                        10.,
                    ),
                    scale: Vec3::new(SPRITE_SCALE.0, SPRITE_SCALE.1, 1.),
                    ..default()
                },
                ..default()
            },
            Player,
            Movable {
                auto_despawn: false,
            },
            Velocity { x: 0.0, y: 0.0 },
            SpriteSize::from(PLAYER_SIZE),
            Name::new("player"),
        ));
        player_state.spawned();
    }
}

fn player_fire_system(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    game_texture: Res<GameTexture>,
    query: Query<&Transform, With<Player>>,
) {
    if let Ok(player_transform) = query.get_single() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            let (x, y) = (
                player_transform.translation.x,
                player_transform.translation.y,
            );
            let x_offset = PLAYER_SIZE.0 / 2.0 * SPRITE_SCALE.0 - 5.0;

            let mut spawn_laser = |x_offset| {
                commands.spawn((
                    SpriteBundle {
                        texture: game_texture.player_laser.clone(),
                        transform: Transform {
                            translation: Vec3::new(x + x_offset, y, 0.),
                            scale: Vec3::new(SPRITE_SCALE.0, SPRITE_SCALE.1, 1.),
                            ..default()
                        },
                        ..default()
                    },
                    Laser,
                    Velocity { x: 0.0, y: 1.0 },
                    Movable { auto_despawn: true },
                    FromPlayer,
                    SpriteSize::from(PLAYER_LASER_SIZE),
                    Name::new("player_laser"),
                ));
            };

            spawn_laser(x_offset);
            spawn_laser(-x_offset);
        }
    }
}

fn player_keyboard_event_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Velocity, With<Player>>,
) {
    if let Ok(mut velocity) = player_query.get_single_mut() {
        velocity.x = if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left)
        {
            -1.
        } else if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
            1.
        } else {
            0.
        };
    }
}

/*
fn player_movement_system(mut query: Query<(&Velocity, &mut Transform), With<Player>>) {
    query.for_each_mut(|(velocity, mut transform)| {
        transform.translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        transform.translation.y += velocity.y * TIME_STEP * BASE_SPEED;
    });
    /*for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
    }*/
}
*/
