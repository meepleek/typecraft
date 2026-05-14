//! Spawn the main level.

use crate::{PrimaryCamera, prelude::*};
use bevy::prelude::*;
use grid::Grid;
use mplk_utils::math::asymptotic_smoothing_with_delta_time;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, track_player);
}

fn track_player(
    grid: Option<Single<&Grid>>,
    trans_q: Query<&GlobalTransform>,
    mut cam_t: Single<&mut Transform, With<PrimaryCamera>>,
    time: Res<Time>,
) {
    let grid = or_return_quiet!(grid);
    let (_, player_e) = or_return!(grid.get_player());
    let player_t = or_return!(trans_q.get(player_e));
    // todo: bounds checking
    cam_t.translation = asymptotic_smoothing_with_delta_time(
        cam_t.translation,
        player_t.translation(),
        0.1,
        time.delta_secs(),
    );
}
