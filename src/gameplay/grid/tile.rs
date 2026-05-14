#![allow(dead_code)]

use bevy::math::U16Vec2;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (move_tile_object, tween_tile_char_alpha));
}

pub const TILE_ALPHA_INACTIVE: f32 = 0.15;
pub const TILE_ALPHA_TARGETABLE: f32 = 1.0;
pub const TILE_ALPHA_HIDDEN: f32 = 0.0;

#[derive(Component, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct TileCoords(pub Coords);

#[derive(Debug, Clone)]
pub struct TargetableTile {
    pub move_char: char,
    pub move_char_e: Entity,
}

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
    Enemy(TypableWord),
    Wall(TypableWord),
}
impl TileObjectKind {
    pub fn enemy(word: impl Into<String>) -> Self {
        TileObjectKind::Enemy(TypableWord::new(word.into().chars().collect::<Vec<_>>()))
    }

    pub fn wall(word: impl Into<String>) -> Self {
        TileObjectKind::Wall(TypableWord::new(word.into().chars().collect::<Vec<_>>()))
    }

    pub fn next_char(&self) -> Option<char> {
        match self {
            Self::Player => None,
            Self::Enemy(word) | Self::Wall(word) => word.next_char(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypableWord {
    pub chars: Vec<char>,
    completed_count: usize,
}
impl TypableWord {
    pub fn new(chars: impl Into<Vec<char>>) -> Self {
        Self {
            chars: chars.into(),
            completed_count: 0,
        }
    }

    pub fn next_char(&self) -> Option<char> {
        self.chars.get(self.completed_count).copied()
    }
}

#[derive(Debug)]
pub struct TargetableNeighbour {
    pub tile: Coords,
    pub targetable: TargetableTile,
    pub object: Option<TileObject>,
}
impl TargetableNeighbour {
    pub fn next_char(&self) -> Option<char> {
        match &self.object {
            Some(to) => to.kind.next_char(),
            None => Some(self.targetable.move_char),
        }
    }
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

fn move_tile_object(
    tile_q: Query<(Entity, &TileCoords), Changed<TileCoords>>,
    grid: Option<Single<&mut grid::Grid>>,
    mut cmd: Commands,
) {
    let mut grid = or_return_quiet!(grid);
    let (_, player_e) = or_return!(grid.get_player());
    for (e, tc) in tile_q {
        let tile = tc.0;
        // also need to fade in/out the from/to move chars

        let world_pos = or_return!(grid.tile_to_world(tile));
        let (start_tile, end_tile) = or_return!(grid.move_entity(player_e, tile));
        // todo: actually the alpha should tween a different entity containing the move char...
        cmd.try_insert_to(
            e,
            TransformPositionLensSrc::new(world_pos).duration(ms(250)),
        );
        cmd.try_insert_to(
            start_tile.move_char_e,
            TextAlphaLensSrc::new(1.).duration(ms(150)),
        );
        cmd.try_insert_to(
            end_tile.move_char_e,
            TextAlphaLensSrc::new(0.).duration(ms(150)),
        );
    }
}

fn tween_tile_char_alpha(
    player_q: Option<Single<&TileCoords, (With<player::Player>, Changed<TileCoords>)>>,
    grid: Option<Single<&mut grid::Grid>>,
    mut cmd: Commands,
) {
    let grid = or_return_quiet!(grid);
    let player_t = or_return_quiet!(player_q).0;
    for (t, tt) in grid.iter_targetable_tiles() {
        let mut alpha = TILE_ALPHA_INACTIVE;
        let dist = (player_t - t).abs();
        if t == player_t {
            alpha = TILE_ALPHA_HIDDEN;
        } else if dist.element_sum() == 1 || (dist.x == dist.y && dist.x == 1) {
            alpha = TILE_ALPHA_TARGETABLE;
        }
        cmd.try_insert_to(
            tt.move_char_e,
            TextAlphaLensSrc::new(alpha).duration(ms(150)),
        );
    }
}
