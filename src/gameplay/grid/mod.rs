pub use crate::prelude::*;
use bevy::math::I16Vec2;

pub mod grid;
pub mod populated;
pub mod template;
pub mod tile;
pub mod world;

pub type Coords = I16Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum TileOrthoDir {
    North,
    East,
    South,
    West,
}
impl TileOrthoDir {
    pub fn from_direction(dir: Coords) -> Option<Self> {
        match dir {
            Coords::NEG_Y => Some(TileOrthoDir::North),
            Coords::X => Some(TileOrthoDir::East),
            Coords::Y => Some(TileOrthoDir::South),
            Coords::NEG_X => Some(TileOrthoDir::West),
            _ => None,
        }
    }

    pub fn direction(&self) -> Coords {
        match self {
            TileOrthoDir::North => Coords::NEG_Y,
            TileOrthoDir::East => Coords::X,
            TileOrthoDir::South => Coords::Y,
            TileOrthoDir::West => Coords::NEG_X,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((grid::plugin, tile::plugin));
}
