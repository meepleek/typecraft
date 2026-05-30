#![allow(dead_code)]

use bevy::math::U16Vec2;
use bevy::platform::collections::HashMap;

use crate::prelude::*;
use player::PlayerGridState;
use tile::*;

pub const DIRS_ORTHO_CW: [Coords; 4] = [Coords::NEG_Y, Coords::X, Coords::Y, Coords::NEG_X];
pub const DIRS_DIAG_CW: [Coords; 4] = [
    Coords::ONE,
    Coords::new(1, -1),
    Coords::NEG_ONE,
    Coords::new(-1, 1),
];
pub const DIRS_CW: [Coords; 8] = [
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
    NotTargetable,
}

#[derive(Debug, PartialEq, Eq, derive_more::Error, derive_more::Display)]
pub enum MoveError {
    Taken,
    OutOfBounds,
    NotTargetable,
    EntityLookupFailed,
}
impl From<PlaceError> for MoveError {
    fn from(place_err: PlaceError) -> Self {
        match place_err {
            PlaceError::Taken => Self::Taken,
            PlaceError::NotTargetable => Self::NotTargetable,
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
    // todo: make this private
    targetable_tiles: HashMap<Coords, TargetableTile>,
    tile_object_coords: HashMap<Entity, Coords>,
}
impl Grid {
    pub const TARGETABLE_TILE_FADE: Duration = Duration::from_millis(150);

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
        if entity == self.player_state.entity {
            return Some(self.player_tile());
        }

        self.tile_object_coords.get(&entity).cloned()
    }

    pub fn is_occupied_tile(&self, tile: Coords) -> bool {
        self.occupied_tiles.contains_key(&tile) || self.player_tile() == tile
    }

    pub fn is_targetable_tile(&self, tile: Coords) -> bool {
        self.targetable_tiles.contains_key(&tile)
    }

    pub fn can_place_at(&self, tile: Coords) -> Result<(), PlaceError> {
        if !self.within_bounds(tile) {
            return Err(PlaceError::OutOfBounds);
        } else if !self.is_targetable_tile(tile) {
            return Err(PlaceError::NotTargetable);
        } else if self.is_occupied_tile(tile) {
            return Err(PlaceError::Taken);
        }
        Ok(())
    }

    pub fn place_object(
        &mut self,
        tile_object: TileObject,
        coords: Coords,
    ) -> Result<(), PlaceError> {
        self.can_place_at(coords)?;
        self.tile_object_coords.insert(tile_object.entity, coords);
        self.occupied_tiles.insert(coords, tile_object);

        Ok(())
    }

    pub fn get_object_mut(&mut self, coords: Coords) -> Option<&mut TileObject> {
        self.occupied_tiles.get_mut(&coords)
    }

    pub fn move_object(
        &mut self,
        object_entity: Entity,
        coords: Coords,
    ) -> Result<(TargetableTile, TargetableTile), MoveError> {
        self.can_place_at(coords)?;
        let Some(prev_tile) = self.tile_object_coords.get(&object_entity).copied() else {
            return Err(MoveError::EntityLookupFailed);
        };
        let Some(tile_obj) = self.clear_tile(prev_tile.clone()) else {
            panic!("Reverse coords lookup failed")
        };
        self.place_object(tile_obj, coords)?;
        let (Some(prev_tt), Some(new_tt)) = (
            self.targetable_tiles.get(&prev_tile),
            self.targetable_tiles.get(&coords),
        ) else {
            panic!("Targetable tile not found");
        };
        Ok((prev_tt.clone(), new_tt.clone()))
    }

    pub fn get_tile_object_or_player_entity(&self, tile: Coords) -> Option<Entity> {
        self.occupied_tiles.get(&tile).map_or_else(
            || {
                if tile == self.player_tile() {
                    Some(self.player_state.entity)
                } else {
                    None
                }
            },
            |obj| Some(obj.entity),
        )
    }

    pub fn clear_tile(&mut self, coords: Coords) -> Option<TileObject> {
        self.occupied_tiles.remove(&coords)
    }

    pub fn get_targetable_tile(&self, tile: Coords) -> Option<&TargetableTile> {
        self.targetable_tiles.get(&tile)
    }

    pub fn targetable_neighbours(
        &self,
        tile: Coords,
        move_dir: TileDirection,
    ) -> impl Iterator<Item = TargetableNeighbour> {
        self.neighbours(tile, move_dir, true).filter_map(move |t| {
            self.targetable_tiles.get(&t).map(|tt| TargetableNeighbour {
                tile: t,
                targetable: tt.clone(),
                object: self.occupied_tiles.get(&t).cloned(),
            })
        })
    }

    pub fn unoccupied_targetable_neighbours(
        &self,
        tile: Coords,
        move_dir: TileDirection,
    ) -> impl Iterator<Item = UnoccupiedTargetableNeighbour> {
        self.targetable_neighbours(tile, move_dir).filter_map(|tn| {
            if tn.tile == self.player_tile() {
                return None;
            }
            match tn.object {
                Some(_) => None,
                None => Some(UnoccupiedTargetableNeighbour {
                    tile: tn.tile,
                    targetable: tn.targetable,
                }),
            }
        })
    }

    pub fn neighbours(
        &self,
        tile: Coords,
        move_dir: TileDirection,
        check_bounds: bool,
    ) -> impl Iterator<Item = Coords> {
        let dirs = Self::neighbour_dirs(move_dir);
        dirs.iter().copied().filter_map(move |dir| {
            let target = tile + dir;
            (!check_bounds || self.within_bounds(target)).then(|| target)
        })
    }

    fn neighbour_dirs(move_dir: TileDirection) -> &'static [Coords] {
        match move_dir {
            TileDirection::Orthogonal => &DIRS_ORTHO_CW,
            TileDirection::Diagonal => &DIRS_DIAG_CW,
            TileDirection::All => &DIRS_CW,
        }
    }

    pub fn iter_tiles(&self) -> TileIterator {
        TileIterator::from_size(self.grid_size)
    }

    pub fn insert_targetable_tile(
        &mut self,
        tile: Coords,
        targetable: TargetableTile,
    ) -> Option<TargetableTile> {
        self.targetable_tiles.insert(tile, targetable)
    }

    pub fn iter_targetable_tiles(&self) -> impl Iterator<Item = (Coords, TargetableTile)> {
        self.iter_tiles()
            .filter_map(|t| self.targetable_tiles.get(&t).map(|tt| (t, tt.clone())))
    }

    pub fn iter_object_tiles(&self) -> impl Iterator<Item = (Coords, &TileObject)> {
        self.iter_tiles()
            .filter_map(|t| self.occupied_tiles.get(&t).map(|to| (t, to)))
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

    pub fn iter_destroyable_wall_tiles(
        &self,
    ) -> impl Iterator<Item = (Coords, Entity, &TypableWords)> {
        self.iter_object_tiles().filter_map(move |(t, to)| {
            let TileObjectKind::Wall(words) = &to.kind else {
                return None;
            };
            Some((t, to.entity, words))
        })
    }

    pub fn is_wall_tile(&self, tile: Coords) -> bool {
        !self.targetable_tiles.contains_key(&tile)
            || matches!(
                self.occupied_tiles.get(&tile),
                Some(TileObject {
                    kind: TileObjectKind::Wall(_),
                    ..
                })
            )
    }

    pub fn targetable_char_alpha(&self, tile: Coords) -> f32 {
        if tile == self.player_tile() {
            return TILE_ALPHA_HIDDEN;
        } else if self.is_player_ortho_tile(tile) {
            return TILE_ALPHA_TARGETABLE;
        }
        TILE_ALPHA_INACTIVE
    }

    #[cfg(test)]
    pub fn test_grid_from_populated(populated: populated::PopulatedGrid) -> Self {
        let mut e_idx = 0;
        let mut get_e = || {
            e_idx += 1;
            tracing::warn!(e_idx);
            Entity::from_raw_u32(e_idx).unwrap()
        };
        let mut grid = Self::new(
            populated.grid_size,
            populated.tile_size,
            player::PlayerGridState {
                tile: populated.player_tile,
                entity: get_e(),
            },
        );
        grid.targetable_tiles = populated
            .targetable_tiles
            .into_iter()
            .map(|(t, c)| {
                (
                    t,
                    TargetableTile {
                        move_char: c,
                        move_char_e: get_e(),
                    },
                )
            })
            .collect();
        for (t, kind) in populated.occupied_tiles {
            grid.place_object(
                TileObject {
                    entity: get_e(),
                    kind: kind,
                },
                t,
            )
            .expect("Failed to place test grid tile object");
        }
        grid
    }

    #[cfg(test)]
    pub fn set_targetable_tiles(&mut self, targetable_tiles: HashMap<Coords, TargetableTile>) {
        self.targetable_tiles = targetable_tiles;
    }

    #[cfg(test)]
    pub fn ascii_debug_map(
        &self,
        show_move_chars: bool,
        remap_tile: impl Fn(Coords) -> Option<(char, DebugGridTileColor)>,
    ) -> String {
        let size = self.grid_size();
        let mut dbg_map = String::with_capacity(size.element_product() as _);
        let x_axis = (0..self.grid_size.x)
            .map(|i| (i % 10).to_string())
            .collect::<String>();
        let header_style = DebugGridTileColor::Header.prefix();
        dbg_map.push_str(&format!("{header_style} _{}_ \n", &x_axis));
        dbg_map.push_str(&format!("{header_style} 0"));
        let mut prev_y = 0;
        for tile in self.iter_tiles() {
            if tile.y != prev_y {
                prev_y = tile.y;
                dbg_map.push_str(&format!("{header_style}{} ", tile.y - 1));
                dbg_map.push('\n');
                dbg_map.push_str(&format!("{header_style}{:2}", tile.y));
            }
            let (c, col) =
                remap_tile(tile).unwrap_or_else(|| match self.occupied_tiles.get(&tile) {
                    Some(TileObject { kind, .. }) => match kind {
                        TileObjectKind::Enemy => ('*', DebugGridTileColor::Red),
                        TileObjectKind::Wall(_) => ('#', DebugGridTileColor::White),
                        TileObjectKind::Goal => ('G', DebugGridTileColor::White),
                    },
                    None if tile == self.player_tile() => ('@', DebugGridTileColor::Green),
                    None => self.targetable_tiles.get(&tile).map_or(
                        ('■', DebugGridTileColor::Dimmed),
                        |tt| {
                            if show_move_chars {
                                (tt.move_char, DebugGridTileColor::White)
                            } else {
                                ('.', DebugGridTileColor::White)
                            }
                        },
                    ),
                });
            dbg_map.push_str(&col.colored(c));
        }
        dbg_map.push_str(&format!(
            "{header_style}{} \n{header_style} _{}_ ",
            size.y - 1,
            &x_axis
        ));
        dbg_map
    }

    #[cfg(test)]
    pub fn print_ascii_debug_map(&self, show_move_chars: bool) {
        self.print_ascii_debug_map_with_remap(show_move_chars, |_| None);
    }

    #[cfg(test)]
    pub fn print_ascii_debug_map_with_remap(
        &self,
        show_move_chars: bool,
        remap_tile: impl Fn(Coords) -> Option<(char, DebugGridTileColor)>,
    ) {
        println!("{}", self.ascii_debug_map(show_move_chars, remap_tile));
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

    use super::*;

    const TEST_LVL_5X3: &'static str = "
    .....
    .*@G.
    .....
    ";

    const TEST_TILE_SIZE: u16 = 96;
    const TILE_SIZE_F32: f32 = TEST_TILE_SIZE as f32;

    #[test_case(1, 2 => matches Ok(_))]
    #[test_case(4, 2 => matches Ok(_))]
    #[test_case(2, 1 => matches Err(PlaceError::Taken))]
    #[test_case(1, 1 => matches Err(PlaceError::Taken))]
    #[test_case(5, 0 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(0, 3 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(50, 0 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(0, 50 => matches Err(PlaceError::OutOfBounds))]
    fn can_place_at_coords(x: i16, y: i16) -> Result<(), PlaceError> {
        let board = test_grid();
        board.can_place_at((x, y).into())
    }

    #[test]
    fn cannot_place_at_coords_when_taken() {
        let coords: Coords = (0, 0).into();
        let mut board = test_grid();
        board
            .place_object(
                TileObject {
                    kind: TileObjectKind::wall(["wall"]),
                    entity: Entity::PLACEHOLDER,
                },
                coords,
            )
            .expect("Place first piece");

        assert_eq!(board.can_place_at(coords), Err(PlaceError::Taken));
    }

    const PLAYER_TILE: (i16, i16) = (4, 2);

    fn test_grid() -> Grid {
        TestGrid::from_str(TEST_LVL_5X3)
    }
}
