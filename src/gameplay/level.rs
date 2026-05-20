//! Spawn the main level.

use crate::prelude::{input::MoveChars, *};
use bevy::prelude::*;
use grid::Grid;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_grid_gizmos);
}

pub const DEBUG_LVL: &str = "
########.....###
######.WGW....##
#..WW..WWW.....#
...WW..........#
...WW..........#
...WW...........
...WW...........
...##...........
...####...WWW###
.....###WWWW####
.......#######..
..........###...
.........###....
........##......
........WW......
.@......WW..##..
......########..
";

const DEFAULT_TILE_SIZE: u16 = 96;

/// A system that spawns the main level.
pub fn spawn_level(
    mut cmd: Commands,
    wordlist: Res<WordList>,
    move_chars: Res<MoveChars>,
) -> Result {
    let grid_template: template::GridChunkTemplate = DEBUG_LVL.parse()?;
    let mut e_cmd = cmd.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
    ));
    let mut rng = rng();
    let populated_grid = populated::PopulatedGrid::new(
        DEFAULT_TILE_SIZE,
        grid_template,
        &wordlist,
        &move_chars,
        &mut rng,
    );
    populated_grid.spawn(&mut e_cmd);

    Ok(())
}

fn draw_grid_gizmos(mut gizmos: Gizmos, grid: Option<Single<&Grid>>) {
    let grid = or_return_quiet!(grid);
    gizmos
        .grid_2d(
            Isometry2d::default(),
            grid.grid_size().as_uvec2(),
            Vec2::splat(grid.tile_size() as f32),
            Color::BLACK,
        )
        .outer_edges();
}
