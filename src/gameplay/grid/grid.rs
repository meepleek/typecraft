#![allow(dead_code)]

use bevy::math::U16Vec2;
use bevy::platform::collections::HashMap;

use crate::prelude::*;
use tile::*;

pub const TILE_SIZE: u16 = 64;

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

pub fn plugin(app: &mut App) {
    app.add_systems(Update, track_grid_position)
        .add_systems(Last, (add_new_tiles_to_grid, add_new_tile_objects_to_grid));
}

#[derive(Component)]
#[require(Transform)]
pub struct Grid {
    width: u16,
    heigth: u16,
    center_global_position: Vec2,
    tile_entities: HashMap<Coords, Entity>,
    occupied_tiles: HashMap<Coords, TileObject>,
    move_chars: HashSet<char>,
    /// Tiles which can contain TileObjects or be moved into
    /// Coords that are not in this map are unbreakable walls
    targetable_tiles: HashMap<Coords, char>,
    entities: HashMap<Entity, Coords>,
}

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

impl Grid {
    pub fn new(width: u16, heigth: u16) -> Self {
        if width == 0 || heigth == 0 {
            panic!("Invalid dimension - no dimension can be 0");
        }

        let mut grid = Self {
            width,
            heigth,
            tile_entities: HashMap::with_capacity((width * heigth) as usize),
            occupied_tiles: HashMap::default(),
            // todo: default to regular QWERTY instead, but make this configurable
            move_chars: "zarstdhneiokjwfpgcluybvm".chars().collect::<HashSet<_>>(),
            targetable_tiles: HashMap::with_capacity((width * heigth) as usize),
            entities: HashMap::default(),
            center_global_position: Vec2::ZERO,
        };

        let mut rng = rand::rng();
        for t in grid
            .iter_tiles()
            .filter(|t| t.min_element() > 0 && t.x < width as i16 - 1 && t.y < heigth as i16 - 1)
        {
            let neighbour_chars = grid.neighbour_chars(t);
            for _ in 0..100 {
                let c = grid
                    .move_chars
                    .iter()
                    .choose(&mut rng)
                    .expect("Failed to pick random move char");
                if !neighbour_chars.contains(c) {
                    grid.targetable_tiles.insert(t, *c);
                    break;
                }
            }
        }
        grid
    }

    #[allow(dead_code)]
    pub fn world_center(&self) -> Vec2 {
        self.center_global_position
    }

    pub fn start_player_tile(&self) -> Coords {
        (self.grid_size() / 2).as_i16vec2()
    }

    pub fn grid_size(&self) -> U16Vec2 {
        (self.width, self.heigth).into()
    }

    pub fn size(&self) -> Vec2 {
        self.grid_size().as_vec2() * TILE_SIZE as f32
    }

    pub fn add_tile_entity(&mut self, tile: Coords, entity: Entity) -> Result<(), AddTileError> {
        if !self.within_bounds(tile) {
            return Err(AddTileError::OutOfBounds);
        }

        self.tile_entities.insert(tile, entity);
        Ok(())
    }

    pub fn get_tile_entity(&self, tile: Coords) -> Option<Entity> {
        self.tile_entities.get(&tile).cloned()
    }

    pub fn get_player_tile(&self) -> Option<Coords> {
        self.occupied_tiles
            .iter()
            .find(|(_, obj)| obj.kind == TileObjectKind::Player)
            .map(|(tile, _)| *tile)
    }

    pub fn get_tile_object(&self, coords: Coords) -> Option<TileObject> {
        self.occupied_tiles.get(&coords).cloned()
    }

    pub fn entity_to_coords(&self, entity: Entity) -> Option<Coords> {
        self.entities.get(&entity).cloned()
    }

    pub fn world_to_tile(&self, pos: Vec2) -> Option<Coords> {
        // transform world position to board space (like screen space but in tiles)
        let half_size = self.size() / 2.;
        let x = half_size.x - self.center_global_position.x + pos.x;
        let y = half_size.y + self.center_global_position.y - pos.y;
        let pos_on_board = Vec2::new(x, y);
        let coords = (pos_on_board / TILE_SIZE as f32).floor().as_i16vec2();
        if !self.within_bounds(coords) {
            return None;
        }

        Some(coords)
    }

    pub fn tile_to_world(&self, tile: Coords) -> Option<Vec2> {
        if tile.min_element() < 0 || tile.x >= self.width as i16 || tile.y >= self.heigth as i16 {
            return None;
        }

        let half_size = self.size() / 2.;
        let half_tile = TILE_SIZE as f32 / 2.;
        let tile_world = tile.as_vec2() * TILE_SIZE as f32;
        let x = tile_world.x + self.center_global_position.x + half_tile - half_size.x;
        let y = -tile_world.y + self.center_global_position.y - half_tile + half_size.y;
        Some(Vec2::new(x, y))
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
        self.entities.insert(tile_object.entity, coords);
        self.occupied_tiles.insert(coords, tile_object);

        Ok(())
    }

    pub fn move_entity(&mut self, entity: Entity, coords: Coords) -> Result<(), MoveError> {
        self.can_place_at(coords)?;
        match self.entities.get(&entity) {
            Some(prev_tile) => match self.clear_tile(*prev_tile) {
                Some(tile_obj) => self.place_entity(tile_obj, coords)?,
                None => panic!("Reverse coords lookup failed"),
            },
            None => return Err(MoveError::EntityLookupFailed),
        }
        Ok(())
    }

    pub fn clear_tile(&mut self, coords: Coords) -> Option<TileObject> {
        self.occupied_tiles.remove(&coords)
    }

    pub fn within_bounds(&self, tile: Coords) -> bool {
        tile.min_element() >= 0 && tile.x < self.width as _ && tile.y < self.heigth as _
    }

    pub fn neighbour_chars(&self, tile: Coords) -> HashSet<char> {
        // area spanning 2 into each direction to avoid using the same chars for opposite neighbours of a character
        TileIterator::centered(U16Vec2::splat(5))
            .flat_map(|dir| {
                let target = tile + dir;
                if !self.within_bounds(target) {
                    return Vec::new();
                }
                match self.occupied_tiles.get(&target) {
                    Some(tile_obj) => match tile_obj.kind {
                        TileObjectKind::Enemy(ref word) | TileObjectKind::Wall(ref word) => {
                            word.chars().collect()
                        }
                        TileObjectKind::Player => Vec::new(),
                    },
                    None => self
                        .targetable_tiles
                        .get(&target)
                        .map_or_else(|| Vec::new(), |c| vec![*c]),
                }
            })
            .collect()
    }

    fn neighbours(
        &self,
        tile: Coords,
        allowed_occupied_tile: Option<Coords>,
        move_dir: TileDirection,
        rng: &mut impl Rng,
    ) -> Vec<Coords> {
        let dirs = Self::neighbour_dirs(move_dir);
        let mut neighbours: Vec<_> = dirs
            .into_iter()
            .copied()
            .filter_map(|dir| {
                let target = tile + dir;
                if allowed_occupied_tile.is_some_and(|t| t == target) {
                    return Some(target);
                }
                self.can_place_at(target).ok().map(|_| target)
            })
            .collect();
        if !neighbours.is_empty() {
            neighbours.shuffle(rng);
        }
        neighbours
    }

    fn neighbour_dirs(move_dir: TileDirection) -> &'static [Coords] {
        match move_dir {
            TileDirection::Orthogonal => &DIRS_ORTHO,
            TileDirection::Diagonal => &DIRS_DIAG,
            TileDirection::All => &DIRS,
        }
    }

    pub fn iter_tiles(&self) -> TileIterator {
        TileIterator::from_size((self.width, self.heigth))
    }

    pub fn iter_targetable_tiles(&self) -> impl Iterator<Item = (Coords, char)> {
        self.iter_tiles()
            .filter_map(|t| self.targetable_tiles.get(&t).map(|c| (t, *c)))
    }

    pub fn iter_movable_tiles(&self) -> impl Iterator<Item = (Coords, char)> {
        self.iter_targetable_tiles()
            .filter(|(t, _)| !self.occupied_tiles.contains_key(t))
    }

    #[allow(dead_code)]
    pub fn ascii_debug_map(&self) -> String {
        let size = self.grid_size();
        let mut dbg_map = String::with_capacity(size.element_product() as _);
        let x_axis = (0..self.width)
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
                    TileObjectKind::Player => '@',
                    TileObjectKind::Enemy(_) => '*',
                    TileObjectKind::Wall(_) => '#',
                },
                None => self.targetable_tiles.get(&tile).map_or('■', |c| *c),
            });
        }
        dbg_map.push_str(&format!("{}\n _{}_", size.y - 1, &x_axis));
        dbg_map
    }
}

fn track_grid_position(
    mut board_q: Query<(&mut Grid, &GlobalTransform), Changed<GlobalTransform>>,
) {
    for (mut board, t) in &mut board_q {
        board.center_global_position = t.translation().truncate();
    }
}

fn add_new_tiles_to_grid(
    entity_q: Query<(Entity, &TileCoords), Added<TileCoords>>,
    mut grid: Single<&mut Grid>,
) {
    for (e, tile) in entity_q {
        grid.add_tile_entity(tile.0, e).expect("invalid tile");
    }
}

fn add_new_tile_objects_to_grid(
    entity_q: Query<(Entity, &TileObjectKind, &GlobalTransform), Added<TileObjectKind>>,
    mut grid: Single<&mut Grid>,
) {
    for (e, kind, t) in entity_q {
        let tile = or_return!(grid.world_to_tile(t.translation().truncate()));
        or_return!(grid.place_entity(
            TileObject {
                entity: e,
                kind: kind.clone(),
            },
            tile,
        ));
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;

    #[test_case(0., 0., 0., 0. => Some(Coords::ONE))]
    #[test_case(64.,-64., 0., 0. => Some(Coords::ZERO))]
    #[test_case(64.,-64., 20., -20. => Some(Coords::ZERO))]
    #[test_case(64.,-64., 40., -40. => Some(Coords::ONE))]
    #[test_case(64., -64., 64., 0. => Some(Coords::new(1, 0)))]
    #[test_case(0., 0., 120., 0. => None)]
    #[test_case(0., 0., -128., 0. => None)]
    #[test_case(0., 0., 0., 120. => None)]
    #[test_case(0., 0., 0., -128. => None)]
    #[traced_test]
    fn world_to_tile(map_x: f32, map_y: f32, world_x: f32, world_y: f32) -> Option<Coords> {
        let mut board = Grid::new(3, 3);
        board.center_global_position = Vec2::new(map_x, map_y);

        board.world_to_tile(Vec2::new(world_x, world_y))
    }

    #[test_case(0., 0., 0, 0 => Some(Vec2::new(-64., 64.)))]
    #[test_case(0., 0., 1, 1 => Some(Vec2::new(0., 0.)))]
    // todo: fix failing test
    // #[test_case(64.,-64., 0, 0 => Some(Vec2::new(64., -64.)))]
    #[test_case(64.,-64., 2, 2 => Some(Vec2::new(128., -128.)))]
    #[test_case(0.,0., 3, 0 => None)]
    #[test_case(0.,0., 0, 3 => None)]
    #[traced_test]
    fn tile_to_world(map_x: f32, map_y: f32, tile_x: i16, tile_y: i16) -> Option<Vec2> {
        let mut board = Grid::new(3, 3);
        board.center_global_position = Vec2::new(map_x, map_y);

        board.tile_to_world(Coords::new(tile_x, tile_y))
    }

    #[test_case(0, 0 => matches Ok(_))]
    #[test_case(3, 3 => matches Ok(_))]
    #[test_case(4, 6 => matches Ok(_))]
    #[test_case(6, 0 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(0, 9 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(50, 0 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(0, 50 => matches Err(PlaceError::OutOfBounds))]
    fn can_place_at_coords(x: i16, y: i16) -> Result<(), PlaceError> {
        let board = Grid::new(6, 9);
        board.can_place_at((x, y).into())
    }

    #[test]
    fn cannot_place_at_coords_when_taken() {
        let coords: Coords = (3, 3).into();
        let mut board = Grid::new(6, 6);
        board
            .place_entity(
                TileObject {
                    kind: TileObjectKind::Player,
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
        let board = Grid::new(size, size);
        board.within_bounds(tile.into())
    }

    const PLAYER_TILE: (i16, i16) = (4, 2);

    /// test map:
    ///
    /// \_01234\_
    /// 0.!...0
    /// 1.###.1
    /// 2..!.@2
    /// 3.....3
    /// 4...!.4
    /// \_01234\_
    fn test_grid() -> Grid {
        let mut grid = Grid::new(5, 5);
        for tile in [(1, 1), (2, 1), (3, 1)] {
            _ = grid
                .place_entity(
                    TileObject {
                        entity: Entity::PLACEHOLDER,
                        kind: TileObjectKind::Wall("Wall".to_string()),
                    },
                    tile.into(),
                )
                .expect("Failed to place an obstacle");
        }
        for tile in [(1, 0), (2, 2), (3, 4)] {
            _ = grid
                .place_entity(
                    TileObject {
                        entity: Entity::PLACEHOLDER,
                        kind: TileObjectKind::Enemy("smite".to_string()),
                    },
                    tile.into(),
                )
                .expect("Failed to place an enemy");
        }
        _ = grid
            .place_entity(
                TileObject {
                    entity: Entity::PLACEHOLDER,
                    kind: TileObjectKind::Player,
                },
                PLAYER_TILE.into(),
            )
            .expect("Failed to place the player");

        println!("{}", grid.ascii_debug_map());
        grid
    }
}
