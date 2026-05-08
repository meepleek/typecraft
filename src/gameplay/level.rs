//! Spawn the main level.

use crate::prelude::*;
use bevy::prelude::*;
use grid::Grid;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_grid_gizmos);
    app.add_observer(draw_tile_chars);
}

/// A system that spawns the main level.
pub fn spawn_level(mut commands: Commands) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![Grid::new(12, 12)],
    ));
}

pub fn draw_tile_chars(_ev: On<Add, Grid>, grid: Single<&Grid>, mut cmd: Commands) {
    let player_tile = grid.grid_size().as_i16vec2() / 2;
    cmd.spawn((
        player::player(),
        Transform::from_translation(grid.tile_to_world(player_tile).unwrap().extend(0.)),
    ));

    for (t, c) in grid.iter_movable_tiles().filter(|(t, _)| *t != player_tile) {
        cmd.spawn((
            Transform::from_translation(grid.tile_to_world(t).unwrap().extend(0.)),
            Text2d::new(c),
            TextFont::from_font_size(40.),
            // todo: use relationships? GridTile(grid_e)
        ));
    }
}

pub fn draw_grid_gizmos(mut gizmos: Gizmos, grid: Option<Single<&Grid>>) {
    let grid = or_return_quiet!(grid);
    gizmos
        .grid_2d(
            Isometry2d::default(),
            // Isometry2d::from_translation(Vec2::splat(tile_size / 2.)),
            UVec2::splat(grid.grid_size().x as u32),
            Vec2::splat(grid::TILE_SIZE as f32),
            Color::BLACK,
        )
        .outer_edges();
}
