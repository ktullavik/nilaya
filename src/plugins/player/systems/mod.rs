use bevy::{prelude::*, input::mouse::MouseMotion, time::Stopwatch, ecs::event::ManualEventReader};
use bevy_rapier3d::prelude::*;

use crate::plugins::world::ChunkQueue;
use crate::plugins::world::systems::enque_chunk;
use crate::{CHUNK_WIDTH, plugins::world::WorldMap};

use super::components::{Player, PlayerCamera, JumpDuration};
use super::super::world::chunk::components::BlockType;

#[derive(Resource, Default)]
pub struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

pub fn movement_system(
    mut player_query: Query<(&mut Velocity, &Player), Without<PlayerCamera>>,
    camera_query: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut velocity, player) = player_query.single_mut();
    let camera_transform = camera_query.single();

    let mut x_axis = 0.0;
    if keyboard.pressed(KeyCode::D) {
        x_axis = 1.0;
    }
    if keyboard.pressed(KeyCode::A) {
        x_axis = -1.0;
    }

    let mut z_axis = 0.0;
    if keyboard.pressed(KeyCode::S) {
        z_axis = 1.0;
    }
    if keyboard.pressed(KeyCode::W) {
        z_axis = -1.0;
    }   

    let movement_input = Quat::from_rotation_y(camera_transform.rotation.to_euler(EulerRot::YXZ).0) * Vec3::new(x_axis, 0.0, z_axis);
    let movement_direction = movement_input.normalize_or_zero();

    velocity.linvel = Vec3::new(movement_direction.x * player.speed * time.delta_seconds(), velocity.linvel.y, movement_direction.z * player.speed * time.delta_seconds());
}

pub fn jump_system(
    time: Res<Time>,
    mut player_query: Query<(&mut JumpDuration, &mut Velocity, &Player)>,
    kbd: Res<Input<KeyCode>>,
) {
    // assume we have exactly one player that jumps with Spacebar
    let (mut jump, mut velocity, player) = player_query.single_mut();

    if kbd.just_pressed(KeyCode::Space) {
        jump.time.reset();
    }

    if kbd.pressed(KeyCode::Space) && jump.time.elapsed_secs() < 0.15 {
        jump.time.tick(time.delta());
        velocity.linvel.y = player.jump_force;
    }
}

pub fn camera_rotation_system(
    windows_query: Query<&Window>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
) {
    let player_transform = player_query.single();

    let (mut camera_transform, mut player_camera) = camera_query.single_mut();
    if let Ok(window) = windows_query.get_single()
    {
        for ev in state.reader_motion.read(&motion) {
            let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
    
            // Using smallest of height or width ensures equal vertical and horizontal sensitivity
            let window_scale = window.height().min(window.width());
            pitch -= (0.0006 * ev.delta.y * window_scale).to_radians();
            yaw -= (0.0006 * ev.delta.x * window_scale).to_radians();
    
            pitch = pitch.clamp(-1.57, 1.57);
    
            // Order is important to prevent unintended roll
            camera_transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }

    player_camera.focus = player_transform.translation;
    camera_transform.translation = player_camera.focus + Vec3::new(0.0, 1.0, 0.0);
}

pub fn player_setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let player = commands.spawn(PbrBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 96.0, 0.0),
            ..Default::default()
        },
        ..default()
    })
    .id();

    commands.entity(player)
        .insert(Player { speed: 400.0, jump_force: 9.0 })
        .insert(JumpDuration { time: Stopwatch::new()})
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED_Z | LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Y)
        .insert(Collider::capsule_y(0.05, 0.4))
        .insert(ColliderMassProperties::Density(1.0))
        .insert(Velocity {
            linvel: Vec3::new(0.0, 0.0, 0.0),
            angvel: Vec3::new(0.0, 0.0, 0.0),
        })
        .insert(GravityScale(6.0))
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled());
    commands.insert_resource(InputState::default());

    commands.spawn(ImageBundle {
        image: asset_server.load("cursor.png").into(),
        style: Style {
            width: Val::Px(16.),
            height: Val::Px(16.),
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            ..default()
        },
        ..default()
    });
}

pub fn block_breaking_system(
    camera_query: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
    rapier_context: Res<RapierContext>,
    mut world_map: ResMut<WorldMap>,
    buttons: Res<Input<MouseButton>>,
    mut chunk_queue: ResMut<ChunkQueue>,
) {
    let camera_transform = camera_query.single();

    if buttons.just_pressed(MouseButton::Left) {
        let origin = camera_transform.translation;
        let direction: Vec3 = camera_transform.forward();
    
        if let Some((_, intersection)) = rapier_context.cast_ray_and_get_normal(origin, direction, 10.0, true, QueryFilter::exclude_dynamic()) {
            let hit = (intersection.point - intersection.normal * 0.5).floor();
            let chunk_pos = ((hit.x / CHUNK_WIDTH as f32).floor() as i32, (hit.z / CHUNK_WIDTH as f32).floor() as i32);
            let (x, y, z) = ((hit.x  - (chunk_pos.0 as f32 * 16.0)) as usize,
                                              (hit.y) as usize,
                                              (hit.z - (chunk_pos.1 as f32 * 16.0)) as usize);

            if world_map.chunks[&chunk_pos][x][y][z] != BlockType::Air{
                world_map.chunks.get_mut(&chunk_pos).unwrap()[x][y][z] = BlockType::Air;
            }

            enque_chunk(&mut chunk_queue, chunk_pos);
            // this is cringe, TODO: rework
            enque_chunk(&mut chunk_queue, (chunk_pos.0-1, chunk_pos.1));
            enque_chunk(&mut chunk_queue, (chunk_pos.0+1, chunk_pos.1));
            enque_chunk(&mut chunk_queue, (chunk_pos.0, chunk_pos.1-1));
            enque_chunk(&mut chunk_queue, (chunk_pos.0, chunk_pos.1+1));
        }
    }
}

pub fn block_placing_system(
    camera_query: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
    rapier_context: Res<RapierContext>,
    mut world_map: ResMut<WorldMap>,
    buttons: Res<Input<MouseButton>>,
    mut chunk_queue: ResMut<ChunkQueue>,
) {
    let camera_transform = camera_query.single();

    if buttons.just_pressed(MouseButton::Right) {
        let origin = camera_transform.translation;
        let direction: Vec3 = camera_transform.forward();
    
        if let Some((_, intersection)) = rapier_context.cast_ray_and_get_normal(origin, direction, 10.0, true, QueryFilter::exclude_dynamic()) {
            let hit = (intersection.point + intersection.normal * 0.5).floor();
            let chunk_pos = ((hit.x / CHUNK_WIDTH as f32).floor() as i32, (hit.z / CHUNK_WIDTH as f32).floor() as i32);
            let (x, y, z) = ((hit.x  - (chunk_pos.0 as f32 * 16.0)) as usize,
                                              (hit.y) as usize,
                                              (hit.z - (chunk_pos.1 as f32 * 16.0)) as usize);

            if world_map.chunks[&chunk_pos][x][y][z] == BlockType::Air {
                world_map.chunks.get_mut(&chunk_pos).unwrap()[x][y][z] = BlockType::Stone;
            }

            enque_chunk(&mut chunk_queue, chunk_pos);
            // this is cringe, TODO: rework
            enque_chunk(&mut chunk_queue, (chunk_pos.0-1, chunk_pos.1));
            enque_chunk(&mut chunk_queue, (chunk_pos.0+1, chunk_pos.1));
            enque_chunk(&mut chunk_queue, (chunk_pos.0, chunk_pos.1-1));
            enque_chunk(&mut chunk_queue, (chunk_pos.0, chunk_pos.1+1));
        }
    }
}

pub fn lock_cursor(
        mut windows_query: Query<&mut Window>,
) {
    if let Ok(mut window) = windows_query.get_single_mut() {
        let cur_pos = Vec2::new(window.width() / 2., window.height() / 2.);
		window.set_cursor_position(Some(cur_pos));
        window.cursor.visible = false;
    }
}