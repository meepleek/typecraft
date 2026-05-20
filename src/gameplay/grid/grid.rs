#![allow(dead_code)]

use bevy::math::U16Vec2;
use bevy::platform::collections::HashMap;

use crate::prelude::*;
use player::PlayerGridState;
use tile::*;

pub const DIRS_ORTHO: [Coords; 4] = [Coords::NEG_Y, Coords::X, Coords::Y, Coords::NEG_X];
pub const DIRS_DIAG: [Coords; 4] = [
    Coords::ONE,
    Coords::new(1, -1),
    Coords::NEG_ONE,
    Coords::new(-1, 1),
];
pub const DIRS: [Coords; 8] = [
    Coords::NEG_Y,
    Coords::new(1, -1),
    Coords::X,
    Coords::ONE,
    Coords::Y,
    Coords::new(-1, 1),
    Coords::NEG_X,
    Coords::NEG_ONE,
];

pub fn plugin(_app: &mut App) {}

#[derive(Debug, PartialEq, Eq, derive_more::Error, derive_more::Display)]
pub enum AddTileError {
    OutOfBounds,
}

#[derive(Debug, PartialEq, Eq, derive_more::Error, derive_more::Display)]
pub enum PlaceError {
    Taken,
    OutOfBounds,
}

#[derive(Debug, PartialEq, Eq, derive_more::Error, derive_more::Display)]
pub enum MoveError {
    Taken,
    OutOfBounds,
    EntityLookupFailed,
}
impl From<PlaceError> for MoveError {
    fn from(place_err: PlaceError) -> Self {
        match place_err {
            PlaceError::Taken => Self::Taken,
            PlaceError::OutOfBounds => Self::OutOfBounds,
        }
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct Grid {
    grid_size: U16Vec2,
    tile_size: u16,
    player_state: PlayerGridState,
    occupied_tiles: HashMap<Coords, TileObject>,
    /// Tiles which can contain TileObjects or be moved into
    /// Coords that are not in this map are unbreakable walls
    pub targetable_tiles: HashMap<Coords, TargetableTile>,
    tile_object_coords: HashMap<Entity, Coords>,
}
impl Grid {
    pub fn new(
        grid_size: impl Into<U16Vec2>,
        tile_size: u16,
        player_state: PlayerGridState,
    ) -> Self {
        let grid_size = grid_size.into();
        if grid_size.min_element() == 0 {
            panic!("Invalid dimensions - no dimension can be 0");
        }

        Self {
            grid_size,
            tile_size,
            player_state,
            occupied_tiles: HashMap::default(),
            targetable_tiles: HashMap::with_capacity((grid_size.element_product()) as usize),
            tile_object_coords: HashMap::default(),
        }
    }

    pub fn player_state(&self) -> &PlayerGridState {
        &self.player_state
    }

    pub fn move_player(&mut self, tile: Coords) -> Coords {
        let prev_tile = self.player_tile();
        self.player_state.tile = tile;
        prev_tile
    }

    pub fn entity_to_coords(&self, entity: Entity) -> Option<Coords> {
        self.tile_object_coords.get(&entity).cloned()
    }

    pub fn can_place_at(&self, coords: Coords) -> Result<(), PlaceError> {
        if !self.within_bounds(coords) {
            return Err(PlaceError::OutOfBounds);
        } else if self.occupied_tiles.contains_key(&coords) {
            return Err(PlaceError::Taken);
        }
        Ok(())
    }

    pub fn place_entity(
        &mut self,
        tile_object: TileObject,
        coords: Coords,
    ) -> Result<(), PlaceError> {
        self.can_place_at(coords)?;
        self.tile_object_coords.insert(tile_object.entity, coords);
        self.occupied_tiles.insert(coords, tile_object);

        Ok(())
    }

    pub fn move_entity(
        &mut self,
        entity: Entity,
        coords: Coords,
    ) -> Result<(TargetableTile, TargetableTile), MoveError> {
        self.can_place_at(coords)?;
        let Some(prev_tile) = self.tile_object_coords.get(&entity).copied() else {
            return Err(MoveError::EntityLookupFailed);
        };
        let Some(tile_obj) = self.clear_tile(prev_tile.clone()) else {
            panic!("Reverse coords lookup failed")
        };
        self.place_entity(tile_obj, coords)?;
        let (Some(prev_tt), Some(new_tt)) = (
            self.targetable_tiles.get(&prev_tile),
            self.targetable_tiles.get(&coords),
        ) else {
            panic!("Targetable tile not found");
        };
        Ok((prev_tt.clone(), new_tt.clone()))
    }

    pub fn clear_tile(&mut self, coords: Coords) -> Option<TileObject> {
        self.occupied_tiles.remove(&coords)
    }

    pub fn targetable_neighbours(
        &self,
        tile: Coords,
        move_dir: TileDirection,
    ) -> impl Iterator<Item = TargetableNeighbour> {
        self.neighbours(tile, move_dir).filter_map(move |t| {
            self.targetable_tiles.get(&t).map(|tt| TargetableNeighbour {
                tile: t,
                targetable: tt.clone(),
                object: self.occupied_tiles.get(&t).cloned(),
            })
        })
    }

    pub fn neighbours(
        &self,
        tile: Coords,
        move_dir: TileDirection,
    ) -> impl Iterator<Item = Coords> {
        let dirs = Self::neighbour_dirs(move_dir);
        dirs.iter().copied().filter_map(move |dir| {
            let target = tile + dir;
            self.within_bounds(target).then(|| target)
        })
    }

    fn neighbour_dirs(move_dir: TileDirection) -> &'static [Coords] {
        match move_dir {
            TileDirection::Orthogonal => &DIRS_ORTHO,
            TileDirection::Diagonal => &DIRS_DIAG,
            TileDirection::All => &DIRS,
        }
    }

    pub fn iter_tiles(&self) -> TileIterator {
        TileIterator::from_size(self.grid_size)
    }

    pub fn iter_targetable_tiles(&self) -> impl Iterator<Item = (Coords, TargetableTile)> {
        self.iter_tiles()
            .filter_map(|t| self.targetable_tiles.get(&t).map(|tt| (t, tt.clone())))
    }

    pub fn iter_movable_tiles(
        &self,
        include_player_tile: bool,
    ) -> impl Iterator<Item = (Coords, TargetableTile)> {
        let player_tile = self.player_tile();
        self.iter_targetable_tiles().filter(move |(t, _)| {
            !self.occupied_tiles.contains_key(t) || (include_player_tile && player_tile == *t)
        })
    }

    #[allow(dead_code)]
    pub fn ascii_debug_map(&self) -> String {
        let size = self.grid_size();
        let mut dbg_map = String::with_capacity(size.element_product() as _);
        let x_axis = (0..self.grid_size.x)
            .map(|i| (i % 10).to_string())
            .collect::<String>();
        dbg_map.push_str(&format!(" _{}_\n", &x_axis));
        dbg_map.push_str(" 0");
        let mut prev_y = 0;
        for tile in self.iter_tiles() {
            if tile.y != prev_y {
                prev_y = tile.y;
                dbg_map.push_str(&format!("{}", tile.y - 1));
                dbg_map.push('\n');
                dbg_map.push_str(&format!("{:2}", tile.y));
            }
            dbg_map.push(match self.occupied_tiles.get(&tile) {
                Some(TileObject { kind, .. }) => match kind {
                    TileObjectKind::Enemy(_) => '*',
                    TileObjectKind::Wall(_) => '#',
                    TileObjectKind::Goal => 'G',
                },
                None if tile == self.player_tile() => '@',
                None => self
                    .targetable_tiles
                    .get(&tile)
                    .map_or('■', |tt| tt.move_char),
            });
        }
        dbg_map.push_str(&format!("{}\n _{}_", size.y - 1, &x_axis));
        dbg_map
    }
}

impl GridSize for Grid {
    fn grid_size(&self) -> U16Vec2 {
        self.grid_size
    }

    fn tile_size(&self) -> u16 {
        self.tile_size
    }

    fn player_tile(&self) -> Coords {
        self.player_state.tile
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;

    const TEST_TILE_SIZE: u16 = 96;
    const TILE_SIZE_F32: f32 = TEST_TILE_SIZE as f32;

    #[test_case(0, 0 => matches Ok(_))]
    #[test_case(3, 3 => matches Ok(_))]
    #[test_case(4, 6 => matches Ok(_))]
    #[test_case(6, 0 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(0, 9 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(50, 0 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(0, 50 => matches Err(PlaceError::OutOfBounds))]
    fn can_place_at_coords(x: i16, y: i16) -> Result<(), PlaceError> {
        let board = test_grid((6, 9));
        board.can_place_at((x, y).into())
    }

    #[test]
    fn cannot_place_at_coords_when_taken() {
        let coords: Coords = (3, 3).into();
        let mut board = test_grid((6, 6));
        board
            .place_entity(
                TileObject {
                    kind: TileObjectKind::wall(["wall"]),
                    entity: Entity::PLACEHOLDER,
                },
                coords,
            )
            .expect("Place first piece");

        assert_eq!(board.can_place_at(coords), Err(PlaceError::Taken));
    }

    #[test_case(3, (0, 0) => true)]
    #[test_case(3, (0, 2) => true)]
    #[test_case(3, (2, 2) => true)]
    #[test_case(3, (1, 1) => true)]
    #[test_case(3, (3, 0) => false)]
    #[test_case(3, (0, 3) => false)]
    #[test_case(3, (-1, 0) => false)]
    #[test_case(3, (0, -1) => false)]
    #[traced_test]
    fn within_bounds(size: u16, tile: (i16, i16)) -> bool {
        let board = test_grid(U16Vec2::splat(size));
        board.within_bounds(tile.into())
    }

    const PLAYER_TILE: (i16, i16) = (4, 2);

    fn test_grid(grid_size: impl Into<U16Vec2>) -> Grid {
        Grid::new(
            grid_size,
            TEST_TILE_SIZE,
            PlayerGridState {
                tile: Coords::ZERO,
                entity: Entity::PLACEHOLDER,
            },
        )
    }
}
