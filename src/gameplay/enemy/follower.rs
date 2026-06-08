use crate::prelude::*;
use bevy::color::palettes::tailwind;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, Follower::run_step.run_on_turn_timer());
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FollowerStep {
    Move(Coords),
    Rotate(TileDir),
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Follower {
    direction: TileDir,
}
impl Follower {
    pub fn bundle(direction: TileDir, transform: Transform) -> impl Bundle {
        let follower = Self { direction };
        (
            super::Enemy,
            object::ContactDamage { dmg: 1 },
            follower,
            transform.with_rotation(direction.rotation().to_quat()),
            Text2d::new("!"),
            TextFont::from_font_size(70.),
            TextColor(tailwind::RED_400.with_alpha(1.).into()),
        )
    }

    fn step(&mut self, current_tile: Coords, grid: &grid::Grid) -> Option<FollowerStep> {
        use FollowerStep::*;
        use pathfinding::directed::astar::astar;

        fn valid_target_tile(grid: &grid::Grid, tile: Coords) -> bool {
            !grid.is_wall_tile(tile) && !grid.is_occupied_tile(tile, true)
        }

        let player_tile = grid.player_tile();
        if player_tile.manhattan_distance(current_tile) > 5 {
            return None;
        }

        astar(
            &current_tile,
            |n| {
                grid.neighbours(*n, tile::TileDirection::Orthogonal, true)
                    .filter(|t| valid_target_tile(grid, *t))
                    .map(|t| (t, 1))
            },
            |n| player_tile.manhattan_distance(*n),
            |n| *n == player_tile,
        )
        .and_then(|path| {
            path.0
                .iter()
                .nth(1) // skip the start tile
                .cloned()
        })
        .map(|next_tile| {
            let dir = TileDir::from_direction(next_tile - current_tile)
                .expect("Failed to get a neighbour dir");
            if dir == self.direction {
                Move(next_tile)
            } else {
                Rotate(dir)
            }
        })
    }

    fn run_step(
        grid: Option<Single<&grid::Grid>>,
        mut enemy_q: Query<(Entity, &mut Follower)>,
        mut cmd: Commands,
    ) {
        use FollowerStep::*;

        let grid = or_return_quiet!(grid);
        for (e, mut follower) in &mut enemy_q {
            let start_tile = or_continue!(grid.entity_to_coords(e));
            let step = follower.step(start_tile, &grid);
            match step {
                Some(Move(end_tile)) => {
                    cmd.trigger(object::ObjectMove {
                        entity: e,
                        start_tile,
                        end_tile,
                    });
                }
                Some(Rotate(dir)) => {
                    follower.direction = dir;
                    cmd.spawn(
                        TransformRotationDegreesLensSrc::new(dir.rotation().as_degrees())
                            .duration(ms(350))
                            .target(e),
                    );
                }
                None => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;
    use FollowerStep::*;
    use TileDir::*;
    use TileOrthoDir::*;

    #[test_case(
        "
        ..WW
        ....
        ###.
        G.@.
        ",
        ((0, 0), North),
        vec![
            Some(Rotate(Ortho(East))),
            Some(Move(Coords::new(1, 0))),
            Some(Rotate(Ortho(South))),
            Some(Move(Coords::new(1, 1))),
            Some(Rotate(Ortho(East))),
            Some(Move(Coords::new(2, 1))),
            Some(Move(Coords::new(3, 1))),
            Some(Rotate(Ortho(South))),
            Some(Move(Coords::new(3, 2))),
            Some(Move(Coords::new(3, 3))),
            Some(Rotate(Ortho(West))),
            Some(Move(Coords::new(2, 3))),
            None
        ] ; "winding path 1"
    )]
    #[test_case(
        "..........@G",
        ((0, 0), East),
        vec![
            None
        ] ; "too far"
    )]
    #[test_case(
        "
        ....
        ####
        G.@.
        ",
        ((0, 0), East),
        vec![
            None
        ] ; "blocked off"
    )]
    #[traced_test]
    fn step(
        lvl: &str,
        initial_state: ((i16, i16), TileOrthoDir),
        expected_steps: Vec<Option<FollowerStep>>,
    ) {
        if expected_steps.len() < 1 {
            panic!("There should be at least 2 steps, otherwise the testcase wouldn't do anything")
        }

        let mut grid = TestGrid::from_str(lvl);
        let entity = Entity::PLACEHOLDER;
        let mut tile = initial_state.0.into();
        grid.place_object(
            tile::TileObject {
                entity,
                kind: tile::TileObjectKind::Enemy,
            },
            tile,
            false,
        )
        .expect("Failed to place enemy");
        let mut follower = Follower {
            direction: Ortho(initial_state.1),
        };

        for expected_step in expected_steps {
            let actual_step = follower.step(tile, &grid);
            if expected_step != actual_step {
                tracing::warn!(?expected_step, ?actual_step, ?follower, ?tile);
                grid.print_ascii_debug_map(false);
            }

            pretty_assertions::assert_eq!(expected_step, actual_step);

            match actual_step {
                Some(Move(next_tile)) => {
                    grid.move_object(entity, next_tile, true)
                        .expect("Failed to move enemy");
                    tile = next_tile;
                }
                Some(Rotate(dir)) => {
                    follower.direction = dir;
                }
                None => {}
            }
        }
    }
}
