use bevy::{prelude::Plugin, app::{Startup, Update}};

use self::systems::{movement_system, jump_system, camera_rotation_system, player_setup_system, block_breaking_system, lock_cursor, block_placing_system};

pub(crate) mod systems;
pub(crate) mod components;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, player_setup_system)
            .add_systems(Update, (movement_system, jump_system, camera_rotation_system, lock_cursor))
            .add_systems(Update, (block_breaking_system, block_placing_system));
    }
}