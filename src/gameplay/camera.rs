//! Spawn the main level.

use crate::prelude::{player::PlayerMoved, *};
use bevy::prelude::*;
use bevy_firefly::{app::FireflyPlugin, data::FireflyConfig, prelude::Occluder2d};
use grid::Grid;
use mplk_utils::math::asymptotic_smoothing_with_delta_time;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((TraumaPlugin, FireflyPlugin));
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, track_player);
    // app.add_systems(Update, occluder_test);
    app.add_observer(update_occluder_z);
}

#[derive(Component)]
pub struct PrimaryCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        PrimaryCamera,
        Shake::default(),
        FireflyConfig {
            // ambient_brightness: 0.2,
            // light_bands: Some(0.25),
            soft_shadows: false,
            // z_sorting: false,
            ..default()
        },
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

fn update_occluder_z(
    _ev: On<PlayerMoved>,
    grid: Option<Single<&Grid>>,
    mut occluder_q: Query<(&Transform, &mut Occluder2d)>,
) {
    let grid = or_return_quiet!(grid);
    for (t, mut occluder) in &mut occluder_q {
        let tile = or_continue!(grid.world_to_tile(t.translation.truncate()));
        if grid.tile_in_player_line_of_sight(tile) {
            occluder.opacity = 1.;
            occluder.z_sorting = true;
        } else {
            occluder.opacity = 0.;
            occluder.z_sorting = false;
        }

        // // todo: this is bad
        // // try to do a simple visibility check
        // // if the tile is visible from the player, then high z, otherwise low
        // t.translation.z = (100 - grid.chess_distance_to_player(tile)) as _;
    }
}

// todo: this doesn't do anything?
// fn occluder_test(mut occluder_q: Query<&mut Occluder2d>, time: Res<Time>) {
//     let opacity = (time.elapsed_secs().sin() + 1.) / 2.;
//     for mut occluder in &mut occluder_q {
//         occluder.opacity = opacity;
//     }
// }
