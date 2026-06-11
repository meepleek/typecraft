#![allow(dead_code)]

use bevy::math::U16Vec2;
use bevy::platform::collections::HashMap;
use itertools::Itertools;

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
        self.tile_object_coords
            .get(&entity)
            .copied()
            .or_else(|| (entity == self.player_state.entity).then(|| self.player_tile()))
    }

    pub fn is_occupied_tile(&self, tile: Coords, allow_player_collision: bool) -> bool {
        self.occupied_tiles.contains_key(&tile)
            || (!allow_player_collision && self.player_tile() == tile)
    }

    pub fn is_targetable_tile(&self, tile: Coords) -> bool {
        self.targetable_tiles.contains_key(&tile)
    }

    pub fn can_place_at(
        &self,
        tile: Coords,
        allow_player_collision: bool,
    ) -> Result<(), PlaceError> {
        if !self.within_bounds(tile) {
            return Err(PlaceError::OutOfBounds);
        } else if !self.is_targetable_tile(tile) {
            return Err(PlaceError::NotTargetable);
        } else if self.is_occupied_tile(tile, allow_player_collision) {
            return Err(PlaceError::Taken);
        }
        Ok(())
    }

    pub fn place_object(
        &mut self,
        tile_object: TileObject,
        coords: Coords,
        allow_player_collision: bool,
    ) -> Result<(), PlaceError> {
        self.can_place_at(coords, allow_player_collision)?;
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
        allow_player_collision: bool,
    ) -> Result<(TargetableTile, TargetableTile), MoveError> {
        self.can_place_at(coords, allow_player_collision)?;
        let Some(prev_tile) = self.entity_to_coords(object_entity) else {
            return Err(MoveError::EntityLookupFailed);
        };
        let Some(tile_obj) = self.clear_tile(prev_tile) else {
            panic!("Reverse coords lookup failed")
        };
        self.place_object(tile_obj, coords, allow_player_collision)?;
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

    pub fn clear_object_tile(&mut self, entity: Entity) -> Option<TileObject> {
        self.entity_to_coords(entity)
            .and_then(|tile| self.clear_tile(tile))
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

    // fn visible_typable_wall_tiles(&self) -> HashSet<Coords> {
    //     use pathfinding::directed::dijkstra::dijkstra_partial;
    //     const MAX_DIST: i16 = 10;

    //     fn reachable_tiles(grid: &Grid, dir_a: Coords, dir_b: Coords) -> HashSet<Coords> {
    //         let player_tile = grid.player_tile();
    //         dijkstra_partial(
    //             &player_tile,
    //             |n| {
    //                 [n + dir_a, n + dir_b]
    //                     .into_iter()
    //                     .filter_map(|t| (!grid.is_wall_tile(t)).then_some((t, 1)))
    //             },
    //             |n| player_tile.manhattan_distance(*n) > MAX_DIST as _,
    //         )
    //         .0
    //         .into_keys()
    //         .collect()
    //     }

    //     let player_tile = self.player_tile();
    //     let ne = reachable_tiles(self, TileDir::NORTH, TileDir::EAST);
    //     let se = reachable_tiles(self, TileDir::SOUTH, TileDir::EAST);
    //     let sw = reachable_tiles(self, TileDir::SOUTH, TileDir::WEST);
    //     let nw = reachable_tiles(self, TileDir::NORTH, TileDir::WEST);

    //     self.iter_destroyable_wall_tiles()
    //         .filter_map(|(t, ..)| {
    //             if t.manhattan_distance(player_tile) > MAX_DIST as _ {
    //                 return None;
    //             }
    //             let dir = (t - player_tile).signum();
    //             let tile_n = t + TileDir::NORTH;
    //             let tile_e = t + TileDir::EAST;
    //             let tile_s = t + TileDir::SOUTH;
    //             let tile_w = t + TileDir::WEST;
    //             if match dir {
    //                 TileDir::NORTH => {
    //                     nw.contains(&tile_s)
    //                         || nw.contains(&tile_e)
    //                         || ne.contains(&tile_s)
    //                         || ne.contains(&tile_w)
    //                 }
    //                 TileDir::NORTH_EAST => ne.contains(&tile_s) || ne.contains(&tile_w),
    //                 TileDir::EAST => {
    //                     ne.contains(&tile_s)
    //                         || ne.contains(&tile_w)
    //                         || se.contains(&tile_n)
    //                         || se.contains(&tile_w)
    //                 }
    //                 TileDir::SOUTH_EAST => se.contains(&tile_n) || se.contains(&tile_w),
    //                 TileDir::SOUTH => {
    //                     se.contains(&tile_n)
    //                         || se.contains(&tile_w)
    //                         || sw.contains(&tile_n)
    //                         || sw.contains(&tile_e)
    //                 }
    //                 TileDir::SOUTH_WEST => sw.contains(&tile_n) || sw.contains(&tile_e),
    //                 TileDir::WEST => {
    //                     sw.contains(&tile_n)
    //                         || sw.contains(&tile_e)
    //                         || nw.contains(&tile_s)
    //                         || nw.contains(&tile_e)
    //                 }
    //                 TileDir::NORTH_WEST => nw.contains(&tile_s) || nw.contains(&tile_e),
    //                 _ => unreachable!("Unknown dir"),
    //             } {
    //                 Some(t)
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect()
    // }

    pub fn tile_in_player_line_of_sight(&self, tile: Coords) -> bool {
        self.tiles_in_line_of_sight(self.player_tile(), tile)
    }

    fn tiles_in_line_of_sight(&self, tile_a: Coords, tile_b: Coords) -> bool {
        tile_a.line_to(tile_b).tuple_windows().all(|(a, b)| {
            let dir = (b - a).signum();
            let is_diag = a.manhattan_distance(b) == 2;
            let is_valid_diag = is_diag
                && (!self.is_wall_tile(a + Coords::X * dir.x)
                    || !self.is_wall_tile(a + Coords::Y * dir.y));

            if b == tile_b {
                !is_diag || is_valid_diag
            } else if self.is_wall_tile(b) {
                false
            } else if is_diag {
                is_valid_diag
            } else {
                true
            }
        })

        // let mut current_tile = tile_a;
        // let dir = tile_b - tile_a;
        // let dist = dir.abs();
        // let sign = dir.signum();
        // let mut move_count = Coords::ZERO;

        // loop {
        //     let prev_tile = current_tile;
        //     let prefer_horizontal =
        //         (1 + 2 * move_count.x) * dist.y < (1 + 2 * move_count.y) * dist.x;
        //     let tile_x = current_tile + Coords::X * sign.x;
        //     let tile_y = current_tile + Coords::Y * sign.y;
        //     let x_blocked = self.is_wall_tile(tile_x);
        //     let y_blocked = self.is_wall_tile(tile_y);
        //     if tile_x == tile_b || tile_y == tile_b {
        //         return true;
        //     }
        //     match (prefer_horizontal, x_blocked, y_blocked) {
        //         // all possible movement blocked by wall
        //         (_, true, true) => return false,
        //         // move in the X dir
        //         (true, false, _) | (false, false, true) => {
        //             current_tile = tile_x;
        //             move_count.x += 1;
        //         }
        //         // move in the Y dir
        //         (false, _, false) | (true, true, false) => {
        //             current_tile = tile_y;
        //             move_count.y += 1;
        //         }
        //     }
        //     if prev_tile == current_tile {
        //         // tried to move by a fallback 0 dir
        //         return false;
        //     }
        // }
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
                    kind: kind.into(),
                },
                t,
                false,
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
        board.can_place_at((x, y).into(), false)
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
                false,
            )
            .expect("Place first piece");

        assert_eq!(board.can_place_at(coords, false), Err(PlaceError::Taken));
    }

    const PLAYER_TILE: (i16, i16) = (4, 2);

    #[test_case(
        "
            WWW
            ...
            .@.
            G..
        ",
        vec![
            (0, 0),
            (1, 0),
            (2, 0),
        ] ; "north wall")]
    #[test_case(
        "
            W.G
            W..
            W@.
            W..
            W..
        ",
        vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
        ] ; "west upclose")]
    #[test_case(
        "
            .WWW.
            .WWW.
            .....
            G.@..
        ",
        vec![
            (1, 1),
            (2, 1),
            (3, 1),
        ] ; "hidden tiles")]
    #[test_case(
        "
            .W@W.
            .WWW.
            .....
            G....
        ",
        vec![
            (1, 0),
            (3, 0),
            (2, 1),
        ] ; "cubby")]
    #[test_case(
        "
            ...WWWW
            ...WWWW
            .......
            ...WWWW
            G.@WWWW
        ",
        vec![
            (3, 0),
            (3, 1),
            (3, 3),
            (3, 4),
        ] ; "keep going")]
    fn tile_in_player_line_of_sight(lvl: &str, expected_tiles: Vec<(i16, i16)>) {
        let grid = TestGrid::from_str(lvl);
        let actual_tiles = grid
            .iter_destroyable_wall_tiles()
            .filter_map(|(t, ..)| grid.tile_in_player_line_of_sight(t).then(|| t))
            .collect::<Vec<_>>();
        grid.print_ascii_debug_map_with_remap(false, |t| {
            if actual_tiles.contains(&t) {
                Some(('#', DebugGridTileColor::Red))
            } else {
                None
            }
        });
        pretty_assertions::assert_eq!(
            expected_tiles
                .into_iter()
                .map(Into::into)
                .collect::<Vec<Coords>>(),
            actual_tiles
        );
    }

    // #[test_case(
    //     "
    //         WWW
    //         ...
    //         .@.
    //         G..
    //     ",
    //     vec![
    //         (0, 0),
    //         (1, 0),
    //         (2, 0),
    //     ] ; "north wall")]
    // #[test_case(
    //     "
    //         W.G
    //         W..
    //         W@.
    //         W..
    //         W..
    //     ",
    //     vec![
    //         (0, 0),
    //         (0, 1),
    //         (0, 2),
    //         (0, 3),
    //         (0, 4),
    //     ] ; "west upclose")]
    // #[test_case(
    //     "
    //         .WWW.
    //         .WWW.
    //         .....
    //         G.@..
    //     ",
    //     vec![
    //         (1, 1),
    //         (2, 1),
    //         (3, 1),
    //     ] ; "hidden tiles")]
    // #[test_case(
    //     "
    //         .W@W.
    //         .WWW.
    //         .....
    //         G....
    //     ",
    //     vec![
    //         (1, 0),
    //         (3, 0),
    //         (2, 1),
    //     ] ; "cubby")]
    // #[test_case(
    //     "
    //         ...WW
    //         ...WW
    //         .....
    //         ...WW
    //         G.@WW
    //     ",
    //     vec![
    //         (3, 0),
    //         (3, 1),
    //         (3, 3),
    //         (3, 4),
    // ] ; "keep going")]
    // fn visible_typable_wall_tiles(lvl: &str, expected_tiles: Vec<(i16, i16)>) {
    //     let grid = TestGrid::from_str(lvl);
    //     let actual_tiles = grid.visible_typable_wall_tiles();
    //     grid.print_ascii_debug_map(false);
    //     pretty_assertions::assert_eq!(
    //         expected_tiles
    //             .into_iter()
    //             .map(Into::into)
    //             .collect::<HashSet<Coords>>(),
    //         actual_tiles
    //     );
    // }

    fn test_grid() -> Grid {
        TestGrid::from_str(TEST_LVL_5X3)
    }
}
