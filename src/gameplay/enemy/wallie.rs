use crate::prelude::*;
use bevy::{color::palettes::tailwind, time::common_conditions::on_real_timer};
use std::ops::Not;

pub(super) fn plugin(app: &mut App) {
    // app.add_systems(Update, wallie_move.run_if(on_real_timer(ms(1500))));
    app.add_systems(Update, Wallie::run_step.run_if(on_real_timer(ms(500))));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Wallie {
    rot_dir: RotationDirection,
    tile_edge: TileOrthoDir,
}
impl Wallie {
    pub fn new(tile_edge: TileOrthoDir, rot_dir: RotationDirection) -> Self {
        Wallie { rot_dir, tile_edge }
    }

    pub fn from_origin_tile(grid: &grid::Grid, tile: Coords, rng: &mut impl Rng) -> Option<Self> {
        grid.can_place_at(tile).ok()?;
        let dir = grid::DIRS_ORTHO_CW
            .into_iter()
            .filter(|dir| grid.is_wall_tile(tile + dir))
            .choose(rng);
        dir.and_then(TileOrthoDir::from_direction).map(|edge| {
            let rot_dir = if rng.random::<bool>() {
                RotationDirection::Clockwise
            } else {
                RotationDirection::CounterClockwise
            };
            Self::new(edge, rot_dir)
        })
    }

    pub fn bundle(tile: Coords, grid: &grid::Grid, rng: &mut impl Rng) -> Option<impl Bundle> {
        let wallie = Self::from_origin_tile(grid, tile, rng)?;
        Some((
            super::Enemy,
            ObjectCoords(tile),
            wallie,
            Text2d::new(template::TemplateTileKind::ENEMY),
            TextFont::from_font_size(90.),
            TextColor(tailwind::RED_400.with_alpha(1.).into()),
        ))
    }

    fn step(&mut self, current_tile: Coords, grid: &grid::Grid) -> Coords {
        use {RotationDirection::*, TileOrthoDir::*};

        fn step_impl(
            wallie: &mut Wallie,
            current_tile: Coords,
            grid: &grid::Grid,
            has_flipped: bool,
        ) -> Coords {
            let (target_dir, wall_dir) = match (wallie.rot_dir, wallie.tile_edge) {
                (Clockwise, North) => (Coords::X, Coords::new(1, -1)),
                (Clockwise, East) => (Coords::Y, Coords::ONE),
                (Clockwise, South) => (Coords::NEG_X, Coords::new(-1, 1)),
                (Clockwise, West) => (Coords::NEG_Y, Coords::NEG_ONE),
                (CounterClockwise, North) => (Coords::NEG_X, Coords::NEG_ONE),
                (CounterClockwise, East) => (Coords::NEG_Y, Coords::ONE),
                (CounterClockwise, West) => (Coords::Y, Coords::new(-1, 1)),
                (CounterClockwise, South) => (Coords::X, Coords::ONE),
            };
            let target_tile = current_tile + target_dir;
            let wall_tile = current_tile + wall_dir;
            let target_tile_valid =
                grid.is_targetable_tile(target_tile) && !grid.is_occupied_tile(target_tile);
            let wall_tile_valid = grid.is_wall_tile(wall_tile);
            match (target_tile_valid, wall_tile_valid) {
                (true, true) => {
                    // continue to next tile, same edge
                    target_tile
                }
                (true, false) => {
                    let prev_edge = wallie.tile_edge;
                    wallie.tile_edge = match wallie.rot_dir {
                        Clockwise => wallie.tile_edge.rotate_ccw(),
                        CounterClockwise => wallie.tile_edge.rotate_cw(),
                    };
                    // actually this doesn't work as there are ambiguities
                    // e.g. East => North can go to 2 different souths
                    // but maybe that's fine given the previous checks
                    // so this just needs to check the appropriate change
                    // or maybe another check is required to determine when
                    // wallie is going round in the same corner
                    current_tile
                        + match (prev_edge, wallie.tile_edge) {
                            (North, East) => Coords::NEG_ONE,
                            (East, North) => Coords::ONE,
                            (South, East) => Coords::new(-1, 1),
                            (East, South) => Coords::new(1, -1),
                            (South, West) => Coords::ONE,
                            (West, South) => Coords::NEG_ONE,
                            (North, West) => Coords::new(1, -1),
                            (West, North) => Coords::NEG_ONE,
                            (North, North)
                            | (North, South)
                            | (East, East)
                            | (East, West)
                            | (South, South)
                            | (South, North)
                            | (West, West)
                            | (West, East) => unreachable!("Invalid round corner rotation"),
                        }
                }
                (false, _) => {
                    if grid.is_wall_tile(target_tile) {
                        wallie.tile_edge = match wallie.rot_dir {
                            Clockwise => wallie.tile_edge.rotate_cw(),
                            CounterClockwise => wallie.tile_edge.rotate_ccw(),
                        };
                        current_tile
                    }
                    // todo: also check for dead-ends here or is that handled by the previous branch?
                    else if !has_flipped {
                        // can't continue - turn around
                        wallie.rot_dir = !wallie.rot_dir;
                        // rerun the whole thing
                        step_impl(wallie, current_tile, grid, true)
                    } else {
                        tracing::error!(?wallie, "failed to step even after flipping");
                        current_tile
                    }
                }
            }
        }

        step_impl(self, current_tile, grid, false)
    }

    fn run_step(
        grid: Option<Single<&grid::Grid>>,
        mut enemy_q: Query<(&mut ObjectCoords, &mut Wallie)>,
    ) {
        let grid = or_return_quiet!(grid);
        for (mut coords, mut walle) in &mut enemy_q {
            let prev_tile = coords.0;
            let tile = walle.step(coords.0, &grid);
            if prev_tile != tile {
                coords.0 = tile;
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Reflect)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}
impl Not for RotationDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            RotationDirection::Clockwise => RotationDirection::CounterClockwise,
            RotationDirection::CounterClockwise => RotationDirection::Clockwise,
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;
    use TileOrthoDir::*;

    const TEST_CTOR_LVL: &'static str = "
    ..#
    ...
    ...
    ...
    @G.
";

    #[test_case(
        Coords::ZERO,
        Some(Wallie {
            rot_dir: RotationDirection::Clockwise,
            tile_edge:TileOrthoDir::West
        }))]
    #[test_case(
        Coords::X,
        Some(Wallie {
            rot_dir: RotationDirection::Clockwise,
            tile_edge:TileOrthoDir::East
        }))]
    #[test_case(
        Coords::new(0, 2),
        Some(Wallie {
            rot_dir: RotationDirection::CounterClockwise,
            tile_edge:TileOrthoDir::West
        }))]
    #[test_case(
        Coords::new(2, 2),
        Some(Wallie {
            rot_dir: RotationDirection::CounterClockwise,
            tile_edge:TileOrthoDir::East
        }))]
    #[test_case(Coords::new(2, 0), None)]
    #[test_case(Coords::new(0, 4), None)]
    #[test_case(Coords::new(1, 2), None)]
    #[traced_test]
    fn from_origin_tile(tile: Coords, expected: Option<Wallie>) {
        let grid = TestGrid::from_str(TEST_CTOR_LVL);

        let actual = Wallie::from_origin_tile(&grid, tile, &mut TestGrid::seeded_rng());

        pretty_assertions::assert_eq!(expected, actual);
    }

    const TEST_STEP_LVL: &'static str = "
    .WWW
    ..WW
    ....
    ###.
    @G##
";

    // todo: Wallie teleported [1, 3] => [3, 1] instead of going around, first movedg from
    // [1, 1]
    const TEST_STEP_CORNER_LVL: &str = "
    ###.
    #.#.
    ..##
    .#..
    ....
    ..@G
    ";

    #[test_case(
        TEST_STEP_LVL,
        RotationDirection::Clockwise,
        vec![
            ((0, 0), East),
            ((1, 1), North),
            ((1, 1), East),
            ((2, 2), North),
            ((3, 2), North),
            ((3, 2), East),
            ((3, 3), East),
            ((3, 3), South),
            ((3, 3), West),
            ((2, 2), South),
            ((1, 2), South),
            ((0, 2), South),
            ((0, 2), West),
            ((0, 1), West),
            ((0, 0), West),
            ((0, 0), North),
        ] ; "Clockwise"
    )]
    #[test_case(
        TEST_STEP_LVL,
        RotationDirection::CounterClockwise,
        vec![
            ((0, 0), West),
            ((0, 1), West),
            ((0, 2), West),
            ((0, 2), South),
            ((1, 2), South),
            ((2, 2), South),
            ((3, 3), West),
            ((3, 3), South),
            ((3, 3), East),
            ((3, 2), East),
            ((3, 2), North),
            ((2, 2), North),
            ((1, 1), East),
            ((1, 1), North),
            ((0, 0), East),
            ((0, 0), North),
        ] ; "CounterClockwise"
    )]
    #[test_case(
        TEST_STEP_CORNER_LVL,
        RotationDirection::Clockwise,
        vec![
            ((1, 1), East),
            ((1, 2), East),
            ((1, 2), South),
            ((0, 3), East),
            ((1, 4), North),
            ((2, 3), West),
            ((2, 3), North),
            ((3, 3), North),
            ((3, 3), East),
            ((3, 4), East),
        ] ; "Corner"
    )]
    #[traced_test]
    fn step(lvl: &str, rot_dir: RotationDirection, steps: Vec<((i16, i16), TileOrthoDir)>) {
        if steps.len() < 2 {
            panic!("There should be at least 2 steps, otherwise the testcase wouldn't do anything")
        }

        let mut grid = TestGrid::from_str(lvl);
        let entity = Entity::PLACEHOLDER;
        let (initial_tile, initial_tile_edge) = steps.first().unwrap();
        grid.place_object(
            tile::TileObject {
                entity,
                kind: tile::TileObjectKind::Enemy,
            },
            initial_tile.clone().into(),
        )
        .expect("Failed to place enemy");
        let mut wallie = Wallie {
            rot_dir,
            tile_edge: *initial_tile_edge,
        };

        for ((current_tile, current_edge), (next_tile, next_edge)) in
            steps.into_iter().tuple_windows()
        {
            let current_tile = Coords::from(current_tile);
            let next_tile = Coords::from(next_tile);

            let actual_next_tile = wallie.step(current_tile, &grid);

            let expected = (next_tile, next_edge);
            let actual = (actual_next_tile, wallie.tile_edge);
            if expected != actual {
                tracing::warn!(?current_tile, ?current_edge, "---current---\n");
                tracing::warn!(?next_tile, ?next_edge, "---next---\n");

                grid.print_ascii_debug_map(false);
            }

            pretty_assertions::assert_eq!(expected, actual);

            if current_tile != next_tile {
                grid.move_object(entity, next_tile)
                    .expect("Failed to move enemy");
            }
        }
    }
}
