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
        use TileOrthoDir::*;

        match dir {
            Coords::NEG_Y => Some(North),
            Coords::X => Some(East),
            Coords::Y => Some(South),
            Coords::NEG_X => Some(West),
            _ => None,
        }
    }

    pub fn direction(&self) -> Coords {
        use TileOrthoDir::*;

        match self {
            North => Coords::NEG_Y,
            East => Coords::X,
            South => Coords::Y,
            West => Coords::NEG_X,
        }
    }

    pub fn rotate_cw(&self) -> Self {
        use TileOrthoDir::*;

        match self {
            North => East,
            East => South,
            South => West,
            West => North,
        }
    }

    pub fn rotate_ccw(&self) -> Self {
        use TileOrthoDir::*;

        match self {
            North => West,
            East => North,
            South => East,
            West => South,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((grid::plugin, tile::plugin));
}
