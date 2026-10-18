use crate::setup::INITIAL_BRICKS_DIRECTION;
use crate::setup::{setup_background, spawn_brick};
use bevy::prelude::*;

// Add the game's entities to our world
pub(crate) fn setup_one_brick(mut commands: Commands, asset_server: Res<AssetServer>) {
    setup_background(&mut commands, asset_server);
    spawn_brick(
        &mut commands,
        Vec2::ZERO,
        INITIAL_BRICKS_DIRECTION.normalize() * 200.0,
    );
}
