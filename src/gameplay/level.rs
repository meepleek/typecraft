//! Spawn the main level.

use bevy::prelude::*;

use crate::{gameplay::player::player, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_grid);
}

/// A system that spawns the main level.
pub fn spawn_level(mut commands: Commands) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![player(),],
    ));
}

pub fn draw_grid(mut gizmos: Gizmos) {
    let tile_size = 64.;
    gizmos.grid_2d(
        Isometry2d::from_translation(Vec2::splat(tile_size / 2.)),
        UVec2::splat(32),
        Vec2::splat(tile_size),
        Color::BLACK,
    );
}
