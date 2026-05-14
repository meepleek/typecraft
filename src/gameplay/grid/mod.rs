pub use crate::prelude::*;
use bevy::math::I16Vec2;

pub mod grid;
pub mod template;
pub mod tile;

pub type Coords = I16Vec2;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((grid::plugin, tile::plugin));
}
