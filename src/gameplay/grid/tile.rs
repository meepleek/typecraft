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

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum TileObjectKind {
    Player,
    Enemy,
    Wall,
}

pub struct TileIterator {
    grid_size: U16Vec2,
    tile: Coords,
}
impl Iterator for TileIterator {
    type Item = Coords;

    fn next(&mut self) -> Option<Self::Item> {
        let size = self.grid_size.as_i16vec2();
        if self.tile.y >= size.y as i16 {
            None
        } else {
            let next = self.tile;
            self.tile.x += 1;
            if self.tile.x == size.x {
                self.tile = (0, self.tile.y + 1).into();
            }
            Some(next)
        }
    }
}
impl TileIterator {
    pub fn from_size(grid_size: impl Into<U16Vec2>) -> Self {
        Self {
            grid_size: grid_size.into(),
            tile: Coords::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
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
}
