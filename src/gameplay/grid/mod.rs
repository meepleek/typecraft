pub use crate::prelude::*;
use bevy::math::I16Vec2;

pub mod grid;
pub mod object;
pub mod populated;
pub mod template;
pub mod tile;
pub mod world;

pub type Coords = I16Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum TileDir {
    Ortho(TileOrthoDir),
    Diag(TileDiagDir),
}
impl TileDir {
    pub const NORTH: Coords = Coords::NEG_Y;
    pub const NORTH_EAST: Coords = Coords::new(1, -1);
    pub const EAST: Coords = Coords::X;
    pub const SOUTH_EAST: Coords = Coords::ONE;
    pub const SOUTH: Coords = Coords::Y;
    pub const SOUTH_WEST: Coords = Coords::new(-1, 1);
    pub const WEST: Coords = Coords::NEG_X;
    pub const NORTH_WEST: Coords = Coords::NEG_ONE;

    pub const DIRS: [TileDir; 8] = {
        use TileDiagDir::*;
        use TileDir::*;
        use TileOrthoDir::*;

        [
            Ortho(North),
            Diag(NorthEast),
            Ortho(East),
            Diag(SouthEast),
            Ortho(South),
            Diag(SouthWest),
            Ortho(West),
            Diag(NorthWest),
        ]
    };

    pub fn from_direction(dir: Coords) -> Option<Self> {
        match dir {
            Self::NORTH | Self::EAST | Self::SOUTH | Self::WEST => Some(Self::Ortho(
                TileOrthoDir::from_direction(dir).expect("Invalid ortho dir"),
            )),
            Self::NORTH_EAST | Self::SOUTH_EAST | Self::SOUTH_WEST | Self::NORTH_WEST => Some(
                Self::Diag(TileDiagDir::from_direction(dir).expect("Invalid diag dir")),
            ),
            _ => None,
        }
    }

    pub fn direction(&self) -> Coords {
        use TileDir::*;

        match self {
            Ortho(tile_ortho_dir) => tile_ortho_dir.direction(),
            Diag(tile_diag_dir) => tile_diag_dir.direction(),
        }
    }

    pub fn rotation(&self) -> Rot2 {
        match self {
            Self::Ortho(tile_ortho_dir) => tile_ortho_dir.rotation(),
            Self::Diag(tile_diag_dir) => tile_diag_dir.rotation(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Hash, strum::EnumIter)]
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
            TileDir::NORTH => Some(North),
            TileDir::EAST => Some(East),
            TileDir::SOUTH => Some(South),
            TileDir::WEST => Some(West),
            _ => None,
        }
    }

    pub fn direction(&self) -> Coords {
        use TileOrthoDir::*;

        match self {
            North => TileDir::NORTH,
            East => TileDir::EAST,
            South => TileDir::SOUTH,
            West => TileDir::WEST,
        }
    }

    pub fn rotation(&self) -> Rot2 {
        Rot2::degrees(match self {
            TileOrthoDir::North => 0.,
            TileOrthoDir::West => 90.,
            TileOrthoDir::South => 180.,
            TileOrthoDir::East => 270.,
        })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Hash, strum::EnumIter)]
pub enum TileDiagDir {
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}
impl TileDiagDir {
    pub fn from_direction(dir: Coords) -> Option<Self> {
        use TileDiagDir::*;

        match dir {
            TileDir::NORTH_EAST => Some(NorthEast),
            TileDir::NORTH_WEST => Some(NorthWest),
            TileDir::SOUTH_EAST => Some(SouthEast),
            TileDir::SOUTH_WEST => Some(SouthWest),
            _ => None,
        }
    }

    pub fn direction(&self) -> Coords {
        use TileDiagDir::*;

        match self {
            NorthEast => TileDir::NORTH_EAST,
            SouthEast => TileDir::SOUTH_EAST,
            SouthWest => TileDir::SOUTH_WEST,
            NorthWest => TileDir::NORTH_WEST,
        }
    }

    pub fn rotation(&self) -> Rot2 {
        Rot2::degrees(match self {
            TileDiagDir::NorthWest => 45.,
            TileDiagDir::SouthWest => 135.,
            TileDiagDir::SouthEast => 225.,
            TileDiagDir::NorthEast => 315.,
        })
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((grid::plugin, tile::plugin, object::plugin));
}
