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
        children![(
            Grid::new(10, 12),
            Transform::default(),
            Visibility::default(),
        )],
    ));
}

pub fn draw_tile_chars(_ev: On<Add, Grid>, grid: Single<(Entity, &mut Grid)>, mut cmd: Commands) {
    let (grid_e, mut grid) = grid.into_inner();
    let player_tile = grid.grid_size().as_i16vec2() / 2;
    grid.spawn_targetable_tiles(&mut or_return!(cmd.get_entity(grid_e)), player_tile);
    let player_e = cmd
        .spawn((
            player::player(),
            Transform::from_translation(grid.tile_to_world(player_tile).unwrap().extend(0.)),
        ))
        .id();
    grid.place_entity(
        tile::TileObject {
            entity: player_e,
            kind: tile::TileObjectKind::Player,
        },
        player_tile,
    )
    .expect("Failed to place player");
}

pub fn draw_grid_gizmos(mut gizmos: Gizmos, grid: Option<Single<&Grid>>) {
    let grid = or_return_quiet!(grid);
    gizmos
        .grid_2d(
            Isometry2d::default(),
            grid.grid_size().as_uvec2(),
            Vec2::splat(grid::TILE_SIZE as f32),
            Color::BLACK,
        )
        .outer_edges();
}
