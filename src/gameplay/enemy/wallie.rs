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
    orientation: WallOrientation,
    tile_edge: TileOrthoDir,
}
impl Wallie {
    pub fn new(tile_edge: TileOrthoDir, orientation: WallOrientation) -> Self {
        Wallie {
            orientation,
            tile_edge,
        }
    }

    pub fn from_origin_tile(grid: &grid::Grid, tile: Coords, rng: &mut impl Rng) -> Option<Self> {
        grid.can_place_at(tile, false).ok()?;
        let dir = grid::DIRS_ORTHO_CW
            .into_iter()
            .filter(|dir| grid.is_wall_tile(tile + dir))
            .choose(rng);
        dir.and_then(TileOrthoDir::from_direction).map(|edge| {
            let orientation = if rng.random::<bool>() {
                WallOrientation::LeftHandSide
            } else {
                WallOrientation::RightHandSide
            };
            Self::new(edge, orientation)
        })
    }

    pub fn bundle(tile: Coords, grid: &grid::Grid, rng: &mut impl Rng) -> Option<impl Bundle> {
        let wallie = Self::from_origin_tile(grid, tile, rng)?;
        Some((
            super::Enemy,
            object::CustomObjectMovement,
            object::AllowPlayerCollision,
            wallie,
            Text2d::new(template::TemplateTileKind::ENEMY),
            TextFont::from_font_size(90.),
            TextColor(tailwind::RED_400.with_alpha(1.).into()),
        ))
    }

    fn step(&mut self, current_tile: Coords, grid: &grid::Grid) -> Option<Coords> {
        use {TileOrthoDir::*, WallOrientation::*};

        fn step_impl(
            wallie: &mut Wallie,
            current_tile: Coords,
            grid: &grid::Grid,
            has_flipped: bool,
        ) -> Option<Coords> {
            let (target_dir, wall_dir) = match (wallie.orientation, wallie.tile_edge) {
                (LeftHandSide, North) => (Coords::X, Coords::new(1, -1)),
                (LeftHandSide, East) => (Coords::Y, Coords::ONE),
                (LeftHandSide, South) => (Coords::NEG_X, Coords::new(-1, 1)),
                (LeftHandSide, West) => (Coords::NEG_Y, Coords::NEG_ONE),
                (RightHandSide, North) => (Coords::NEG_X, Coords::NEG_ONE),
                (RightHandSide, East) => (Coords::NEG_Y, Coords::ONE),
                (RightHandSide, West) => (Coords::Y, Coords::new(-1, 1)),
                (RightHandSide, South) => (Coords::X, Coords::ONE),
            };
            let target_tile = current_tile + target_dir;
            let wall_tile = current_tile + wall_dir;
            let target_tile_valid =
                grid.is_targetable_tile(target_tile) && !grid.is_occupied_tile(target_tile, true);
            let wall_tile_valid = grid.is_wall_tile(wall_tile);
            match (target_tile_valid, wall_tile_valid) {
                (true, true) => {
                    // continue to next tile, same edge
                    Some(target_tile)
                }
                (true, false) => {
                    let corner_tile = current_tile + wallie.tile_edge.direction();
                    if !grid.is_wall_tile(corner_tile) {
                        return None;
                    }

                    let prev_edge = wallie.tile_edge;
                    wallie.tile_edge = match wallie.orientation {
                        LeftHandSide => wallie.tile_edge.rotate_ccw(),
                        RightHandSide => wallie.tile_edge.rotate_cw(),
                    };
                    tracing::debug!(?target_tile, ?wall_tile, ?prev_edge, edge=?wallie.tile_edge, "turning around");
                    Some(
                        current_tile +
                        // don't need to match on wall orientation 'cause each pair is only valid for one orientation & not reachable for the other
                       match (prev_edge, wallie.tile_edge) {
                            (North, East) => Coords::NEG_ONE,
                            (East, North) => Coords::ONE,
                            (South, East) => Coords::new(-1, 1),
                            (East, South) => Coords::new(1, -1),
                            (South, West) => Coords::ONE,
                            (West, South) => Coords::NEG_ONE,
                            (North, West) => Coords::new(1, -1),
                            (West, North) => Coords::new(-1, 1),
                            (North, North)
                            | (North, South)
                            | (East, East)
                            | (East, West)
                            | (South, South)
                            | (South, North)
                            | (West, West)
                            | (West, East) => unreachable!("Invalid round corner rotation"),
                        },
                    )
                }
                (false, _) => {
                    if grid.is_wall_tile(target_tile) {
                        wallie.tile_edge = match wallie.orientation {
                            LeftHandSide => wallie.tile_edge.rotate_cw(),
                            RightHandSide => wallie.tile_edge.rotate_ccw(),
                        };
                        Some(current_tile)
                    }
                    // todo: also check for dead-ends here or is that handled by the previous branch?
                    else if !has_flipped {
                        // can't continue - turn around
                        wallie.orientation = !wallie.orientation;
                        // rerun the whole thing
                        step_impl(wallie, current_tile, grid, true)
                    } else {
                        tracing::error!(?wallie, "failed to step even after flipping");
                        Some(current_tile)
                    }
                }
            }
        }

        step_impl(self, current_tile, grid, false)
    }

    fn run_step(
        grid: Option<Single<&mut grid::Grid>>,
        mut enemy_q: Query<(Entity, &mut Wallie)>,
        mut cmd: Commands,
    ) {
        let mut grid = or_return_quiet!(grid);
        for (e, mut wallie) in &mut enemy_q {
            let start_tile = or_continue!(grid.entity_to_coords(e));
            let tile = wallie.step(start_tile, &grid);
            match tile {
                Some(end_tile) => {
                    if start_tile != end_tile {
                        // or_continue!(grid.move_object(e, end_tile, true));
                        cmd.trigger(object::ObjectMove {
                            entity: e,
                            start_tile,
                            end_tile,
                        });
                    }
                }
                None => cmd.trigger(object::ObjectExploded {
                    entity: e,
                    tile: start_tile,
                }),
            }
        }
    }
}

/// Which side is the followed wall in relation the the Wallie
#[derive(Debug, PartialEq, Clone, Copy, Reflect)]
pub enum WallOrientation {
    LeftHandSide,
    RightHandSide,
}
impl Not for WallOrientation {
    type Output = Self;

    fn not(self) -> Self::Output {
        use WallOrientation::*;
        match self {
            LeftHandSide => RightHandSide,
            RightHandSide => LeftHandSide,
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
    use WallOrientation::*;

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
            orientation: LeftHandSide,
            tile_edge:TileOrthoDir::West
        }))]
    #[test_case(
        Coords::X,
        Some(Wallie {
            orientation: LeftHandSide,
            tile_edge:TileOrthoDir::East
        }))]
    #[test_case(
        Coords::new(0, 2),
        Some(Wallie {
            orientation: RightHandSide,
            tile_edge:TileOrthoDir::West
        }))]
    #[test_case(
        Coords::new(2, 2),
        Some(Wallie {
            orientation: RightHandSide,
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

    #[test]
    #[traced_test]
    fn from_origin_tile_single_possible_edge() {
        const LVL: &str = "
        ....
        .#..
        ....
        @G..
        ";
        let grid = TestGrid::from_str(LVL);

        for seed in 0..1000 {
            let mut rng = TestGrid::rng_from_seed(seed);

            let wallie = Wallie::from_origin_tile(&grid, (2, 1).into(), &mut rng)
                .expect("failed to get wallie from origin");

            pretty_assertions::assert_eq!(TileOrthoDir::West, wallie.tile_edge);
        }
    }

    const TEST_STEP_LVL: &'static str = "
    .WWW
    ..WW
    ....
    ###.
    @G##
";

    #[test_case(
        TEST_STEP_LVL,
        LeftHandSide,
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
        ] ; "LeftHandSide"
    )]
    #[test_case(
        TEST_STEP_LVL,
        RightHandSide,
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
        ] ; "RightHandSide"
    )]
    #[test_case(
        "
        ###.
        #.#.
        ..##
        .#..
        ....
        ..@G
        ",
        LeftHandSide,
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
    #[test_case(
        "
        ...
        .#.
        ...
        @G.
        ",
        RightHandSide,
        vec![
            ((2, 1), West),
            ((1, 2), North),
            ((0, 1), East),
            ((1, 0), South),
            ((2, 1), West),
        ] ; "single pivot tile - RightHandSide"
    )]
    #[test_case(
        "
        ...
        .#.
        ...
        @G.
        ",
        LeftHandSide,
        vec![
            ((2, 1), West),
            ((1, 0), South),
            ((0, 1), East),
            ((1, 2), North),
            ((2, 1), West),
        ] ; "single pivot tile - LeftHandSide"
    )]
    #[test_case(
        "
        .#
        ##
        @G
        ",
        LeftHandSide,
        vec![
            ((0, 0), North),
            ((0, 0), East),
            ((0, 0), South),
            ((0, 0), West),
            ((0, 0), North),
        ] ; "single tile - LeftHandSide"
    )]
    #[test_case(
        "
        .#
        ##
        @G
        ",
        RightHandSide,
        vec![
            ((0, 0), North),
            ((0, 0), West),
            ((0, 0), South),
            ((0, 0), East),
            ((0, 0), North),
        ] ; "single tile - RightHandSide"
    )]
    #[traced_test]
    fn step(lvl: &str, orientation: WallOrientation, steps: Vec<((i16, i16), TileOrthoDir)>) {
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
            orientation,
            tile_edge: *initial_tile_edge,
        };

        for ((current_tile, current_edge), (next_tile, next_edge)) in
            steps.into_iter().tuple_windows()
        {
            let current_tile = Coords::from(current_tile);
            let next_tile = Coords::from(next_tile);

            let actual_next_tile = wallie.step(current_tile, &grid).expect("Failed to step");

            let expected = (next_tile, next_edge);
            let actual = (actual_next_tile, wallie.tile_edge);
            if expected != actual {
                tracing::warn!(?current_tile, ?current_edge, "---current---\n");
                tracing::warn!(?next_tile, ?next_edge, "---next---\n");

                grid.print_ascii_debug_map(false);
            }

            pretty_assertions::assert_eq!(expected, actual);

            if current_tile != next_tile {
                grid.move_object(entity, next_tile, true)
                    .expect("Failed to move enemy");
            }
        }
    }
}
