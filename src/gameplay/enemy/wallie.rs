use bevy::{color::palettes::tailwind, time::common_conditions::on_real_timer};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // app.add_systems(Update, wallie_move.run_if(on_real_timer(ms(1500))));
    app.add_systems(Update, wallie_move.run_if(on_real_timer(ms(500))));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Wallie {
    anchor: Coords,
    prev_tile: Coords,
    rot_dir: RotationDirection,
    backtracking_dir: Option<TileDir>,
}
impl Wallie {
    pub fn bundle(tile: Coords, anchor: Coords, direction: RotationDirection) -> impl Bundle {
        (
            super::Enemy,
            ObjectCoords(tile),
            Wallie {
                // used as a deny list for movement
                // so using the initial position is fine
                prev_tile: tile,
                anchor,
                rot_dir: direction,
                backtracking_dir: None,
            },
            Text2d::new(template::TemplateTileKind::ENEMY),
            TextFont::from_font_size(90.),
            TextColor(tailwind::RED_400.with_alpha(1.).into()),
        )
    }

    fn step(&mut self, current_tile: Coords, grid: &grid::Grid) -> Coords {
        fn is_deadend_anchor(grid: &grid::Grid, tile: Coords, next_tile: Coords) -> bool {
            grid.unoccupied_targetable_neighbours(tile, tile::TileDirection::All)
                .all(|utn| utn.tile == next_tile)
        }

        let neighbours = grid
            .unoccupied_targetable_neighbours(current_tile, tile::TileDirection::Orthogonal)
            // .inspect(|utn| tracing::warn!(t=?utn.tile, "pre-filter"))
            .filter(|utn| {
                utn.tile != self.prev_tile && utn.tile.chebyshev_distance(self.anchor) == 1
            })
            // .inspect(|utn| tracing::warn!(t=?utn.tile, "post-filter"))
            .collect::<Vec<_>>();

        // todo: actually this needs to check that there are no neighbours whatsoever without filtering out prev_tile
        // if neighbours.len() == 0 {
        //     tracing::warn!(?self, "Wallie can't move");
        //     // todo: explode or smt?
        //     return current_tile;
        // }

        let next_tile = match (neighbours.len(), neighbours.first(), self.backtracking_dir) {
            (0, None, None) => {
                self.backtracking_dir = Some(self.prev_tile - current_tile);
                self.prev_tile
            }
            (0, None, Some(backtracking_dir)) => {
                let next_tile = current_tile + backtracking_dir;
                tracing::warn!(?neighbours, "backtracking\n");
                next_tile
            }
            (_, Some(next_tile), _) => {
                // no longer backtracking
                self.backtracking_dir = None;
                next_tile.tile
            }
            _ => unreachable!(),
        };
        self.prev_tile = current_tile;

        // update anchor
        // wall-hit anchor change
        let dir = next_tile - current_tile;
        let maybe_corner_tile = next_tile + dir;
        if grid.is_wall_tile(maybe_corner_tile)
            && !is_deadend_anchor(grid, maybe_corner_tile, next_tile)
        {
            self.anchor = maybe_corner_tile;
            return next_tile;
        }

        // moving anchor in rotation direction
        // let turning_around_anchor = next_tile.manhattan_distance(self.anchor) == 1;
        let mut possible_anchor_dirs = grid::DIRS_ORTHO_CW;
        let ccw = self.rot_dir == RotationDirection::CounterClockwise;
        if ccw {
            possible_anchor_dirs.reverse();
        }
        let dir_to_check = if self.backtracking_dir.is_some() && ccw {
            -dir
        } else {
            dir
        };
        let dir_i = possible_anchor_dirs
            .iter()
            .position(|d| *d == dir_to_check)
            .expect("Failed to find index of anchor dir");
        possible_anchor_dirs.rotate_left(dir_i);
        tracing::warn!(?possible_anchor_dirs);
        let moved_anchor = possible_anchor_dirs
            .into_iter()
            .filter_map(|d| {
                let tile = if self.backtracking_dir.is_some() && ccw {
                    current_tile
                } else {
                    next_tile
                };
                let anchor_tile = tile + d;
                if !grid.is_wall_tile(anchor_tile)
                    // skip the movement dir, 'cause that's already handled above
                    || d == dir_to_check
                    || tile.manhattan_distance(anchor_tile) != 1
                {
                    return None;
                }
                Some(anchor_tile)
            })
            .next();
        if let Some(moved_anchor) = moved_anchor {
            tracing::warn!(leaving=?self.backtracking_dir, "============\nmoved anchor tile");
            self.anchor = moved_anchor;
            return next_tile;
        }

        next_tile
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Reflect)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

fn wallie_move(
    grid: Option<Single<&grid::Grid>>,
    mut enemy_q: Query<(&mut ObjectCoords, &mut Wallie)>,
) {
    let grid = or_return_quiet!(grid);
    for (mut coords, mut walle) in &mut enemy_q {
        coords.0 = walle.step(coords.0, &grid);
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;

    const TEST_LVL: &'static str = "
..WW
...W
....
####
@G##
";

    #[derive(Debug, PartialEq)]
    struct WallEStepData {
        tile: Coords,
        prev_tile: Coords,
        anchor: Coords,
        backtracking_dir: Option<TileDir>,
    }

    // CW direction
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 1),
          tile: Coords::new(0, 0),
          anchor: Coords::new(0, -1),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(0, 0),
          tile: Coords::new(1, 0),
          anchor: Coords::new(2, 0),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 0),
          tile: Coords::new(1, 0),
          anchor: Coords::new(2, 0),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(1, 0),
          tile: Coords::new(1, 1),
          anchor: Coords::new(2, 0),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 0),
          tile: Coords::new(1, 1),
          anchor: Coords::new(2, 0),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(1, 1),
          tile: Coords::new(2, 1),
          anchor: Coords::new(3, 1),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 1),
          tile: Coords::new(2, 1),
          anchor: Coords::new(3, 1),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(2, 1),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 1),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(3, 2),
          anchor: Coords::new(3, 3),
          backtracking_dir: None
        } ; "CW: entering dead-end"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(3, 2),
          anchor: Coords::new(3, 3),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(3, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3),
          backtracking_dir: Some(TileDir::new(-1, 0))
        } ; "CW: leaving dead-end"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(3, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3),
          backtracking_dir: Some(TileDir::new(-1, 0))
        },
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(1, 2),
          anchor: Coords::new(1, 3),
          backtracking_dir: None
        } ; "CW: left dead-end"
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(1, 2),
          anchor: Coords::new(1, 3),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(1, 2),
          tile: Coords::new(0, 2),
          anchor: Coords::new(-1, 2),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 2),
          tile: Coords::new(0, 2),
          anchor: Coords::new(-1, 2),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(0, 2),
          tile: Coords::new(0, 1),
          anchor: Coords::new(-1, 1),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 2),
          tile: Coords::new(0, 1),
          anchor: Coords::new(-1, 1),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(0, 1),
          tile: Coords::new(0, 0),
          anchor: Coords::new(0, -1),
          backtracking_dir: None
        }
    )]
    // CCW direction
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 0),
          tile: Coords::new(0, 0),
          anchor: Coords::new(-1, 0),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(0, 0),
          tile: Coords::new(0, 1),
          anchor: Coords::new(-1, 1),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 0),
          tile: Coords::new(0, 1),
          anchor: Coords::new(-1, 1),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(0, 1),
          tile: Coords::new(0, 2),
          anchor: Coords::new(0, 3),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 1),
          tile: Coords::new(0, 2),
          anchor: Coords::new(0, 3),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(0, 2),
          tile: Coords::new(1, 2),
          anchor: Coords::new(1, 3),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 2),
          tile: Coords::new(1, 2),
          anchor: Coords::new(1, 3),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(1, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(3, 2),
          anchor: Coords::new(3, 1),
          backtracking_dir: None
        } ; "CCW: entering dead-end"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(3, 2),
          anchor: Coords::new(1, 3),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(3, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(3, 1),
          backtracking_dir: Some(TileDir::new(-1, 0))
        } ; "CCW: leaving dead-end"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(3, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(3, 1),
          backtracking_dir: Some(TileDir::new(-1, 0))
        },
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(2, 1),
          anchor: Coords::new(2, 0),
          backtracking_dir: None
        } ; "CCW: left dead-end"
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(2, 1),
          anchor: Coords::new(2, 0),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(2, 1),
          tile: Coords::new(1, 1),
          anchor: Coords::new(2, 0),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 1),
          tile: Coords::new(1, 1),
          anchor: Coords::new(2, 0),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(1, 1),
          tile: Coords::new(1, 0),
          anchor: Coords::new(1, -1),
          backtracking_dir: None
        }
    )]
    #[test_case(
        RotationDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 1),
          tile: Coords::new(1, 0),
          anchor: Coords::new(1, -1),
          backtracking_dir: None
        },
        WallEStepData {
          prev_tile : Coords::new(1, 0),
          tile: Coords::new(0, 0),
          anchor: Coords::new(-1, 0),
          backtracking_dir: None
        }
    )]
    #[traced_test]
    fn step(rot_dir: RotationDirection, initial_state: WallEStepData, expected: WallEStepData) {
        let mut grid = TestGrid::from_str(TEST_LVL);
        grid.place_object(
            tile::TileObject {
                entity: Entity::PLACEHOLDER,
                kind: tile::TileObjectKind::Enemy,
            },
            initial_state.tile.into(),
        )
        .expect("Failed to place enemy");
        let mut wallie = Wallie {
            rot_dir,
            anchor: initial_state.anchor,
            prev_tile: initial_state.prev_tile,
            backtracking_dir: initial_state.backtracking_dir,
        };
        tracing::warn!(?wallie);
        grid.print_ascii_debug_map(false);

        let next_tile = wallie.step(initial_state.tile.into(), &grid);
        let actual = WallEStepData {
            tile: next_tile,
            prev_tile: wallie.prev_tile,
            anchor: wallie.anchor,
            backtracking_dir: wallie.backtracking_dir,
        };

        pretty_assertions::assert_eq!(expected, actual)
    }
}
