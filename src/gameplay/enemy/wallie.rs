use std::ops::Not;

use bevy::{color::palettes::tailwind, time::common_conditions::on_real_timer};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // app.add_systems(Update, wallie_move.run_if(on_real_timer(ms(1500))));
    app.add_systems(Update, wallie_move.run_if(on_real_timer(ms(500))));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Wallie {
    rot_dir: RotationDirection,
    tile_edge: TileOrthoDir,
}
impl Wallie {
    pub fn bundle(tile: Coords) -> impl Bundle {
        (
            super::Enemy,
            ObjectCoords(tile),
            Wallie {
                // todo:
                rot_dir: RotationDirection::Clockwise,
                // todo:
                tile_edge: TileOrthoDir::North,
            },
            Text2d::new(template::TemplateTileKind::ENEMY),
            TextFont::from_font_size(90.),
            TextColor(tailwind::RED_400.with_alpha(1.).into()),
        )
    }

    fn step(&mut self, current_tile: Coords, grid: &grid::Grid) -> Coords {
        use {RotationDirection::*, TileOrthoDir::*};

        let (target_dir, wall_dir) = match (self.rot_dir, self.tile_edge) {
            (Clockwise, North) => (Coords::X, Coords::new(1, -1)),
            (Clockwise, East) => (Coords::Y, Coords::ONE),
            (Clockwise, South) => (Coords::NEG_X, Coords::new(-1, 1)),
            (Clockwise, West) => (Coords::NEG_Y, Coords::NEG_ONE),
            (CounterClockwise, North) => todo!(),
            (CounterClockwise, East) => todo!(),
            (CounterClockwise, South) => todo!(),
            (CounterClockwise, West) => todo!(),
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
                let prev_edge = self.tile_edge;
                self.tile_edge = match self.rot_dir {
                    Clockwise => self.tile_edge.rotate_ccw(),
                    CounterClockwise => todo!(),
                };
                tracing::warn!(edge=?self.tile_edge, dir=?self.tile_edge.direction(), "rotate around corner");
                // actually this doesn't work as there are ambiguities
                // e.g. East => North can go to 2 different souths
                // but maybe that's fine given the previous checks
                // so this just needs to check the appropriate change
                // or maybe another check is required to determine when
                // wallie is going round in the same corner
                current_tile
                    + match (prev_edge, self.tile_edge) {
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
                    self.tile_edge = match self.rot_dir {
                        Clockwise => self.tile_edge.rotate_cw(),
                        CounterClockwise => todo!(),
                    };
                    current_tile
                }
                // todo: also check for deadends here or is that handled by the previous branch?
                else {
                    // can't continue - turn around
                    self.rot_dir = !self.rot_dir;
                    // rerun the whole thing
                    // todo: add some recursion bool or smt. to prevent stack overflow
                    self.step(current_tile, grid)
                }
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

fn wallie_move(
    grid: Option<Single<&grid::Grid>>,
    mut enemy_q: Query<(&mut ObjectCoords, &mut Wallie)>,
) {
    let grid = or_return_quiet!(grid);
    for (mut coords, mut walle) in &mut enemy_q {
        let tile = walle.step(coords.0, &grid);
        coords.0 = tile;
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;

    const TEST_LVL: &'static str = "
    .WWW
    ..WW
    ....
    ###.
    @G##
";

    #[derive(Debug, PartialEq)]
    struct WallEStepData {
        tile: Coords,
        tile_edge: TileOrthoDir,
    }

    // CW direction
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::North,
        },
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::East,
        } ; "CW: [0, 0] East"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::East,
        },
        WallEStepData {
          tile: Coords::new(1, 1),
          tile_edge: TileOrthoDir::North,
        } ; "CW: [1, 1] South"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(1, 1),
          tile_edge: TileOrthoDir::North,
        },
        WallEStepData {
          tile: Coords::new(1, 1),
          tile_edge: TileOrthoDir::East,
        } ; "CW: [1, 1] East"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(1, 1),
          tile_edge: TileOrthoDir::East,
        },
        WallEStepData {
          tile: Coords::new(2, 2),
          tile_edge: TileOrthoDir::North,
        } ; "CW: [2, 2] North"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(2, 2),
          tile_edge: TileOrthoDir::North,
        },
        WallEStepData {
          tile: Coords::new(3, 2),
          tile_edge: TileOrthoDir::North,
        } ; "CW: [3, 2] North"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(3, 2),
          tile_edge: TileOrthoDir::North,
        },
        WallEStepData {
          tile: Coords::new(3, 2),
          tile_edge: TileOrthoDir::East,
        } ; "CW: [3, 2] East"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(3, 2),
          tile_edge: TileOrthoDir::East,
        },
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::East,
        } ; "CW: [3, 3] East"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::East,
        },
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::South,
        } ; "CW: [3, 3] South"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::South,
        },
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::West,
        } ; "CW: [3, 3] West"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::West,
        },
        WallEStepData {
          tile: Coords::new(2, 2),
          tile_edge: TileOrthoDir::South,
        } ; "CW: [2, 2] South"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(2, 2),
          tile_edge: TileOrthoDir::South,
        },
        WallEStepData {
          tile: Coords::new(1, 2),
          tile_edge: TileOrthoDir::South,
        } ; "CW: [1, 2] South"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(1, 2),
          tile_edge: TileOrthoDir::South,
        },
        WallEStepData {
          tile: Coords::new(0, 2),
          tile_edge: TileOrthoDir::South,
        } ; "CW: [0, 2] South"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(0, 2),
          tile_edge: TileOrthoDir::South,
        },
        WallEStepData {
          tile: Coords::new(0, 2),
          tile_edge: TileOrthoDir::West,
        } ; "CW: [0, 2] West"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(0, 2),
          tile_edge: TileOrthoDir::West,
        },
        WallEStepData {
          tile: Coords::new(0, 1),
          tile_edge: TileOrthoDir::West,
        } ; "CW: [0, 1] West"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(0, 1),
          tile_edge: TileOrthoDir::West,
        },
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::West,
        } ; "CW: [0, 0] West"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::West,
        },
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::North,
        } ; "CW: [0, 0] North"
    )]
    // CCW direction
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::North,
        },
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::West,
        } ; "CCW: [0, 0] West"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::West,
        },
        WallEStepData {
          tile: Coords::new(0, 1),
          tile_edge: TileOrthoDir::West,
        } ; "CCW: [0, 1] West"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(0, 1),
          tile_edge: TileOrthoDir::West,
        },
        WallEStepData {
          tile: Coords::new(0, 2),
          tile_edge: TileOrthoDir::West,
        } ; "CCW: [0, 2] West"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(0, 2),
          tile_edge: TileOrthoDir::West,
        },
        WallEStepData {
          tile: Coords::new(0, 2),
          tile_edge: TileOrthoDir::South,
        } ; "CCW: [0, 2] South"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(0, 2),
          tile_edge: TileOrthoDir::South,
        },
        WallEStepData {
          tile: Coords::new(1, 2),
          tile_edge: TileOrthoDir::South,
        } ; "CCW: [1, 2] South"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(1, 2),
          tile_edge: TileOrthoDir::South,
        },
        WallEStepData {
          tile: Coords::new(2, 2),
          tile_edge: TileOrthoDir::South,
        } ; "CCW: [2, 2] South"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(2, 2),
          tile_edge: TileOrthoDir::South,
        },
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::West,
        } ; "CCW: [3, 3] West"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::West,
        },
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::South,
        } ; "CCW: [3, 3] South"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::South,
        },
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::East,
        } ; "CCW: [3, 3] East"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(3, 3),
          tile_edge: TileOrthoDir::East,
        },
        WallEStepData {
          tile: Coords::new(3, 2),
          tile_edge: TileOrthoDir::East,
        } ; "CCW: [3, 2] East"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(3, 2),
          tile_edge: TileOrthoDir::East,
        },
        WallEStepData {
          tile: Coords::new(3, 2),
          tile_edge: TileOrthoDir::North,
        } ; "CCW: [3, 2] North"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(3, 2),
          tile_edge: TileOrthoDir::North,
        },
        WallEStepData {
          tile: Coords::new(2, 2),
          tile_edge: TileOrthoDir::North,
        } ; "CCW: [2, 2] North"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(2, 2),
          tile_edge: TileOrthoDir::North,
        },
        WallEStepData {
          tile: Coords::new(1, 1),
          tile_edge: TileOrthoDir::East,
        } ; "CCW: [1, 1] East"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(1, 1),
          tile_edge: TileOrthoDir::East,
        },
        WallEStepData {
          tile: Coords::new(1, 1),
          tile_edge: TileOrthoDir::North,
        } ; "CCW: [1, 1] North"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(1, 1),
          tile_edge: TileOrthoDir::North,
        },
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::East,
        } ; "CCW: [0, 0] East"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::East,
        },
        WallEStepData {
          tile: Coords::new(0, 0),
          tile_edge: TileOrthoDir::North,
        } ; "CCW: [0, 0] North"
    )]
    #[traced_test]
    fn step(rot_dir: RotationDirection, initial_state: WallEStepData, expected: WallEStepData) {
        let mut grid = TestGrid::from_str(TEST_LVL);
        let entity = Entity::PLACEHOLDER;
        grid.place_object(
            tile::TileObject {
                entity,
                kind: tile::TileObjectKind::Enemy,
            },
            initial_state.tile.into(),
        )
        .expect("Failed to place enemy");
        let mut wallie = Wallie {
            rot_dir,
            tile_edge: initial_state.tile_edge,
        };

        let next_tile = wallie.step(initial_state.tile, &grid);
        let actual = WallEStepData {
            tile: next_tile,
            tile_edge: wallie.tile_edge,
        };

        grid.print_ascii_debug_map(
            false,
            Some(move |_t| {
                None
                // if t == actual.tile {
                //     Some(('*', DebugGridTileColor::Red))
                // }
                // // else if t == actual.prev_tile {
                // //     let dir = actual.tile - actual.prev_tile;
                // //     Some((
                // //         match (dir.x, dir.y) {
                // //             (0, -1) => '🢁',
                // //             (1, 0) => '🢂',
                // //             (0, 1) => '🢃',
                // //             (-1, 0) => '🡸',
                // //             _ => unreachable!(),
                // //         },
                // //         DebugGridTileColor::White,
                // //     ))
                // // }
                // // else if t == expected.anchor {
                // //     Some(('⨯', DebugGridTileColor::Red))
                // // } else if t == actual.anchor {
                // //     Some(('⨯', DebugGridTileColor::Green))
                // // }
                // else {
                //     None
                // }
            }),
        );

        pretty_assertions::assert_eq!(expected, actual)
    }
}
