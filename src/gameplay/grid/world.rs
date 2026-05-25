use crate::prelude::*;
use bevy::math::U16Vec2;

pub trait GridSize {
    fn grid_size(&self) -> U16Vec2;
    fn tile_size(&self) -> u16;
    fn player_tile(&self) -> Coords;
}

pub trait GridWorld {
    fn world_size(&self) -> Vec2;
    fn within_bounds(&self, tile: Coords) -> bool;
    fn world_to_tile(&self, pos: Vec2) -> Option<Coords>;
    fn tile_to_world(&self, tile: Coords) -> Option<Vec2>;
    fn is_player_ortho_tile(&self, tile: Coords) -> bool;
    fn chess_distance_to_player(&self, tile: Coords) -> u16;
}
impl<G> GridWorld for G
where
    G: GridSize,
{
    fn world_size(&self) -> Vec2 {
        self.grid_size().as_vec2() * self.tile_size() as f32
    }

    fn within_bounds(&self, tile: Coords) -> bool {
        let grid_size = self.grid_size();
        tile.min_element() >= 0 && tile.x < grid_size.x as _ && tile.y < grid_size.y as _
    }

    fn world_to_tile(&self, pos: Vec2) -> Option<Coords> {
        // transform world position to board space (like screen space but in tiles)
        let half_size = self.world_size() / 2.;
        let x = half_size.x + pos.x;
        let y = half_size.y - pos.y;
        let pos_on_board = Vec2::new(x, y);
        let coords = (pos_on_board / self.tile_size() as f32)
            .floor()
            .as_i16vec2();
        if !self.within_bounds(coords) {
            return None;
        }

        Some(coords)
    }

    fn tile_to_world(&self, tile: Coords) -> Option<Vec2> {
        if !self.within_bounds(tile) {
            return None;
        }

        let half_size = self.world_size() / 2.;
        let half_tile = self.tile_size() as f32 / 2.;
        let tile_world = tile.as_vec2() * self.tile_size() as f32;
        let x = tile_world.x + half_tile - half_size.x;
        let y = -tile_world.y - half_tile + half_size.y;
        Some(Vec2::new(x, y))
    }

    fn is_player_ortho_tile(&self, tile: Coords) -> bool {
        self.player_tile().manhattan_distance(tile) <= 1
    }

    fn chess_distance_to_player(&self, tile: Coords) -> u16 {
        self.player_tile().chebyshev_distance(tile)
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;

    const TEST_TILE_SIZE: u16 = 96;
    const TILE_SIZE_F32: f32 = TEST_TILE_SIZE as f32;

    struct TestGrid;
    impl GridSize for TestGrid {
        fn grid_size(&self) -> U16Vec2 {
            (3, 3).into()
        }

        fn tile_size(&self) -> u16 {
            TEST_TILE_SIZE
        }

        fn player_tile(&self) -> Coords {
            Coords::ZERO
        }
    }

    #[test_case((0, 0) => true)]
    #[test_case((0, 2) => true)]
    #[test_case((2, 2) => true)]
    #[test_case((1, 1) => true)]
    #[test_case((3, 0) => false)]
    #[test_case((0, 3) => false)]
    #[test_case((-1, 0) => false)]
    #[test_case((0, -1) => false)]
    #[traced_test]
    fn within_bounds(tile: (i16, i16)) -> bool {
        TestGrid.within_bounds(tile.into())
    }

    #[test_case(0., 0. => Some(Coords::ONE))]
    #[test_case(TILE_SIZE_F32 * 0.4, TILE_SIZE_F32 * -0.4 => Some(Coords::ONE))]
    #[test_case(TILE_SIZE_F32 * 0.6, TILE_SIZE_F32 * -0.6 => Some(Coords::new(2, 2)))]
    #[test_case(TILE_SIZE_F32, 0. => Some(Coords::new(2, 1)))]
    #[test_case(TILE_SIZE_F32 * 1.9 , 0. => None)]
    #[test_case(TILE_SIZE_F32 * -2., 0. => None)]
    #[test_case(0., TILE_SIZE_F32 * 1.9 => None)]
    #[test_case(0., TILE_SIZE_F32 * -2. => None)]
    #[traced_test]
    fn world_to_tile(world_x: f32, world_y: f32) -> Option<Coords> {
        TestGrid.world_to_tile(Vec2::new(world_x, world_y))
    }

    #[test_case(0, 0 => Some(Vec2::new(-TILE_SIZE_F32, TILE_SIZE_F32)))]
    #[test_case(1, 1 => Some(Vec2::new(0., 0.)))]
    #[test_case(2, 2 => Some(Vec2::new(TILE_SIZE_F32, -TILE_SIZE_F32)))]
    #[test_case(3, 0 => None)]
    #[test_case(0, 3 => None)]
    #[traced_test]
    fn tile_to_world(tile_x: i16, tile_y: i16) -> Option<Vec2> {
        TestGrid.tile_to_world(Coords::new(tile_x, tile_y))
    }
}
