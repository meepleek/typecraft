#![allow(dead_code)]

use bevy::math::U16Vec2;

use crate::prelude::*;

pub fn plugin(_app: &mut App) {}

#[derive(Component, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct TileCoords(pub Coords);

// use this as a single source of truth for both the movement & ability direction
// to avoid tricky combos like ortho movement + diag attack that could lead to buggy pathfinding
// this should also simplify the UI & mental overhead for players
#[derive(Component, Debug, Clone, Copy)]
pub enum TileDirection {
    Orthogonal,
    Diagonal,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileObject {
    pub entity: Entity,
    pub kind: TileObjectKind,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum TileObjectKind {
    Player,
    Enemy(String),
    Wall(String),
}

pub struct TileIterator {
    grid_size: U16Vec2,
    tile: Coords,
    start_tile: Coords,
}
impl Iterator for TileIterator {
    type Item = Coords;

    fn next(&mut self) -> Option<Self::Item> {
        let size = self.grid_size.as_i16vec2();
        if self.tile.y - self.start_tile.y >= size.y as i16 {
            None
        } else {
            let next = self.tile;
            self.tile.x += 1;
            if self.tile.x - self.start_tile.x == size.x {
                self.tile = (self.start_tile.x, self.tile.y + 1).into();
            }
            Some(next)
        }
    }
}
impl TileIterator {
    pub fn from_size(grid_size: impl Into<U16Vec2>) -> Self {
        Self::new(Coords::ZERO, grid_size)
    }

    pub fn centered(grid_size: impl Into<U16Vec2>) -> Self {
        let grid_size = grid_size.into();
        Self::new(-grid_size.as_i16vec2() / 2, grid_size)
    }

    fn new(centre: impl Into<Coords>, grid_size: impl Into<U16Vec2>) -> Self {
        let tile = centre.into();
        Self {
            grid_size: grid_size.into(),
            tile,
            start_tile: tile,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_size() {
        let tiles: Vec<_> = TileIterator::from_size((5, 3)).collect();
        assert_eq!(
            tiles,
            [
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (4, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
                (4, 2),
            ]
            .map(Into::into)
        );
    }

    #[test]
    fn centered() {
        let tiles: Vec<_> = TileIterator::centered((5, 3)).collect();
        assert_eq!(
            tiles,
            [
                (-2, -1),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (-2, 1),
                (-1, 1),
                (0, 1),
                (1, 1),
                (2, 1),
            ]
            .map(Into::into)
        );
    }
}
