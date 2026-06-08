use crate::prelude::*;
use bevy::color::palettes::tailwind;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, Follower::run_step.run_on_turn_timer());
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Follower;
//  {
//     direction: TileDir,
// }
impl Follower {
    /* pub fn new(direction: TileDir) -> Self {
        Self { direction }
    } */

    pub fn bundle(transform: Transform) -> impl Bundle {
        let follower = Self;
        (
            super::Enemy,
            object::ContactDamage { dmg: 1 },
            follower,
            transform,
            Text2d::new("+"),
            TextFont::from_font_size(65.),
            TextColor(tailwind::RED_400.with_alpha(1.).into()),
        )
    }

    fn step(&mut self, current_tile: Coords, grid: &grid::Grid) -> Option<Coords> {
        use pathfinding::directed::astar::astar;

        fn valid_target_tile(grid: &grid::Grid, tile: Coords) -> bool {
            !grid.is_wall_tile(tile) && !grid.is_occupied_tile(tile, true)
        }

        let player_tile = grid.player_tile();
        if player_tile.manhattan_distance(current_tile) > 8 {
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
    }

    fn run_step(
        grid: Option<Single<&grid::Grid>>,
        mut enemy_q: Query<(Entity, &mut Follower)>,
        mut cmd: Commands,
    ) {
        let grid = or_return_quiet!(grid);
        for (e, mut follower) in &mut enemy_q {
            let start_tile = or_continue!(grid.entity_to_coords(e));
            let step = follower.step(start_tile, &grid);
            match step {
                Some(end_tile) => {
                    cmd.trigger(object::ObjectMove {
                        entity: e,
                        start_tile,
                        end_tile,
                    });
                }
                None => {}
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use test_case::test_case;
//     use tracing_test::traced_test;

//     use super::*;
//     use FollowerStep::*;
//     use TileDiagDir::*;
//     use TileDir::*;
//     use TileOrthoDir::*;

//     // todo: revamp to use FollowerStep
//     #[test_case(
//         "
//         .WWW
//         ..WW
//         ....
//         ###.
//         @G##
//         ",
//         ((0, 0), Ortho(South)),
//         vec![
//             Move(Coords::new(0, 1)),
//             Move(Coords::new(0, 2)),
//             Rotate(Ortho(North)),
//             Move(Coords::new(0, 1)),
//             Move(Coords::new(0, 0)),
//             Rotate(Ortho(South)),
//         ] ; "vertical"
//     )]
//     #[test_case(
//         "
//         .WWW
//         ..WW
//         ....
//         ###.
//         @G##
//         ",
//         ((0, 1), Ortho(East)),
//         vec![
//             Move(Coords::new(1, 1)),
//             Rotate(Ortho(West)),
//             Move(Coords::new(0, 1)),
//             Rotate(Ortho(East)),
//         ] ; "horizontal"
//     )]
//     #[test_case(
//         "
//         ...
//         ...
//         ...
//         ###
//         @G#
//         ",
//         ((1, 0), Diag(NorthEast)),
//         vec![
//             Rotate(Diag(SouthEast)),
//             Move(Coords::new(2, 1)),
//             Rotate(Diag(SouthWest)),
//             Move(Coords::new(1, 2)),
//             Rotate(Diag(NorthWest)),
//             Move(Coords::new(0, 1)),
//             Rotate(Diag(NorthEast)),
//             Move(Coords::new(1, 0)),
//         ] ; "diag - SE"
//     )]
//     #[traced_test]
//     fn step(lvl: &str, initial_state: ((i16, i16), TileDir), expected_steps: Vec<FollowerStep>) {
//         if expected_steps.len() < 2 {
//             panic!("There should be at least 2 steps, otherwise the testcase wouldn't do anything")
//         }

//         let mut grid = TestGrid::from_str(lvl);
//         let entity = Entity::PLACEHOLDER;
//         let mut tile = initial_state.0.into();
//         grid.place_object(
//             tile::TileObject {
//                 entity,
//                 kind: tile::TileObjectKind::Enemy,
//             },
//             tile,
//             false,
//         )
//         .expect("Failed to place enemy");
//         let mut bouncer = Follower {
//             direction: initial_state.1,
//         };

//         for expected_step in expected_steps {
//             let actual_step = bouncer.step(tile, &grid);

//             if expected_step != actual_step {
//                 tracing::warn!(?expected_step, ?actual_step, ?bouncer, ?tile);
//                 grid.print_ascii_debug_map(false);
//             }

//             pretty_assertions::assert_eq!(expected_step, actual_step);

//             match actual_step {
//                 Move(next_tile) => {
//                     grid.move_object(entity, next_tile, true)
//                         .expect("Failed to move enemy");
//                     tile = next_tile;
//                 }
//                 Rotate(next_dir) => bouncer.direction = next_dir,
//             }
//         }
//     }
// }
