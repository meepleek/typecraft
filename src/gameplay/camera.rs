//! Spawn the main level.

use crate::prelude::*;
use bevy::prelude::*;
use grid::Grid;
use mplk_utils::math::asymptotic_smoothing_with_delta_time;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TraumaPlugin);
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, track_player);
}

#[derive(Component)]
pub struct PrimaryCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        PrimaryCamera,
        Shake::default(),
    ));
}

fn track_player(
    grid: Option<Single<&Grid>>,
    trans_q: Query<&GlobalTransform>,
    mut cam_t: Single<&mut Transform, With<PrimaryCamera>>,
    time: Res<Time>,
) {
    let grid = or_return_quiet!(grid);
    let player_e = grid.player_state().entity;
    let player_t = or_return!(trans_q.get(player_e));
    // todo: bounds checking
    let cam_pos = cam_t.translation;
    let player_pos = player_t.translation();
    let dist_sq = cam_pos.distance_squared(player_pos);
    cam_t.translation = if dist_sq < 100_000. {
        asymptotic_smoothing_with_delta_time(cam_pos, player_pos, 0.1, time.delta_secs())
    } else {
        player_pos
    };
}
