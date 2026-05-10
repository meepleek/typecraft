use bevy::prelude::*;

pub mod grid;
pub mod input;
pub mod level;
pub mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((level::plugin, player::plugin, grid::plugin, input::plugin));
}
