use bevy::{color::palettes::tailwind, time::common_conditions::on_real_timer};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // app.add_systems(Update, wall_e_move.run_if(on_real_timer(ms(1500))));
    app.add_systems(Update, wall_e_move.run_if(on_real_timer(ms(500))));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct WallE {
    anchor: Coords,
    previous_tile: Coords,
    direction: WallieDirection,
}
impl WallE {
    fn step(&mut self, current_tile: Coords, grid: &grid::Grid) -> Coords {
        fn is_deadend_anchor(grid: &grid::Grid, tile: Coords, next_tile: Coords) -> bool {
            grid.unoccupied_targetable_neighbours(tile, tile::TileDirection::All)
                .all(|utn| utn.tile == next_tile)
        }

        let next_tile = match grid
            .unoccupied_targetable_neighbours(current_tile, tile::TileDirection::Orthogonal)
            .filter(|utn| {
                utn.tile != self.previous_tile && utn.tile.chebyshev_distance(self.anchor) == 1
            })
            .next()
        {
            Some(next_tile) => next_tile.tile,
            None => {
                // no matching tile => return back
                self.previous_tile
            }
        };
        self.previous_tile = current_tile;

        // update anchor
        // wall-hit anchor change
        let dir = next_tile - current_tile;
        let maybe_corner_tile = next_tile + dir;
        if grid.is_wall_tile(maybe_corner_tile)
            && !is_deadend_anchor(grid, maybe_corner_tile, next_tile)
        {
            tracing::warn!("============\nmaybe corner anchor");
            self.anchor = maybe_corner_tile;
            return next_tile;
        }

        // moving anchor in rotation direction
        // let turning_around_anchor = next_tile.manhattan_distance(self.anchor) == 1;
        let mut possible_anchor_dirs = grid::DIRS_ORTHO_CW;
        if self.direction == WallieDirection::CounterClockwise {
            possible_anchor_dirs.reverse();
        }
        let dir_i = possible_anchor_dirs
            .iter()
            .position(|d| *d == dir)
            .expect("Failed to find index of anchor dir");
        possible_anchor_dirs.rotate_left(dir_i);
        let moved_anchor = possible_anchor_dirs
            .into_iter()
            .filter_map(|d| {
                let anchor_tile = next_tile + d;
                if !grid.is_wall_tile(anchor_tile)
                    // skip the movement dir, 'cause that's already handled above
                    || d == dir
                    || next_tile.manhattan_distance(anchor_tile) != 1
                {
                    return None;
                }
                Some(anchor_tile)
            })
            .next();
        if let Some(moved_anchor) = moved_anchor {
            tracing::warn!("============\nmoved anchor tile");
            self.anchor = moved_anchor;
            return next_tile;
        }

        next_tile
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Reflect)]
pub enum WallieDirection {
    Clockwise,
    CounterClockwise,
}

pub fn wall_e(tile: Coords, anchor: Coords, direction: WallieDirection) -> impl Bundle {
    (
        super::Enemy,
        ObjectCoords(tile),
        WallE {
            anchor,
            // used as a deny list for movement
            // so using the initial position is fine
            previous_tile: tile,
            direction,
        },
        Text2d::new(template::TemplateTileKind::ENEMY),
        TextFont::from_font_size(90.),
        TextColor(tailwind::RED_400.with_alpha(1.).into()),
    )
}

fn wall_e_move(
    grid: Option<Single<&grid::Grid>>,
    mut enemy_q: Query<(&mut ObjectCoords, &mut WallE)>,
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
    }

    // CW direction
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 1),
          tile: Coords::new(0, 0),
          anchor: Coords::new(0, -1)
        },
        WallEStepData {
          prev_tile : Coords::new(0, 0),
          tile: Coords::new(1, 0),
          anchor: Coords::new(2, 0)
        }
    )]
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 0),
          tile: Coords::new(1, 0),
          anchor: Coords::new(2, 0)
        },
        WallEStepData {
          prev_tile : Coords::new(1, 0),
          tile: Coords::new(1, 1),
          anchor: Coords::new(2, 0)
        }
    )]
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 0),
          tile: Coords::new(1, 1),
          anchor: Coords::new(2, 0)
        },
        WallEStepData {
          prev_tile : Coords::new(1, 1),
          tile: Coords::new(2, 1),
          anchor: Coords::new(3, 1)
        }
    )]
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 1),
          tile: Coords::new(2, 1),
          anchor: Coords::new(3, 1)
        },
        WallEStepData {
          prev_tile : Coords::new(2, 1),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3)
        }
    )]
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 1),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3)
        },
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(3, 2),
          anchor: Coords::new(3, 3)
        } ; "CW: Anchor not set to dead-end tile"
    )]
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(3, 2),
          anchor: Coords::new(3, 3)
        },
        WallEStepData {
          prev_tile : Coords::new(3, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3)
        } ; "CW: Reset prev tile & update anchor when no next tile (dead-end)"
    )]
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(3, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3)
        },
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(1, 2),
          anchor: Coords::new(1, 3)
        }
    )]
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(1, 2),
          anchor: Coords::new(1, 3)
        },
        WallEStepData {
          prev_tile : Coords::new(1, 2),
          tile: Coords::new(0, 2),
          anchor: Coords::new(-1, 2)
        }
    )]
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 2),
          tile: Coords::new(0, 2),
          anchor: Coords::new(-1, 2)
        },
        WallEStepData {
          prev_tile : Coords::new(0, 2),
          tile: Coords::new(0, 1),
          anchor: Coords::new(-1, 1)
        }
    )]
    #[test_case(
        WallieDirection::Clockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 2),
          tile: Coords::new(0, 1),
          anchor: Coords::new(-1, 1)
        },
        WallEStepData {
          prev_tile : Coords::new(0, 1),
          tile: Coords::new(0, 0),
          anchor: Coords::new(0, -1)
        }
    )]
    // CCW direction
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 0),
          tile: Coords::new(0, 0),
          anchor: Coords::new(-1, 0)
        },
        WallEStepData {
          prev_tile : Coords::new(0, 0),
          tile: Coords::new(0, 1),
          anchor: Coords::new(-1, 1)
        }
    )]
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 0),
          tile: Coords::new(0, 1),
          anchor: Coords::new(-1, 1)
        },
        WallEStepData {
          prev_tile : Coords::new(0, 1),
          tile: Coords::new(0, 2),
          anchor: Coords::new(0, 3)
        }
    )]
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 1),
          tile: Coords::new(0, 2),
          anchor: Coords::new(0, 3)
        },
        WallEStepData {
          prev_tile : Coords::new(0, 2),
          tile: Coords::new(1, 2),
          anchor: Coords::new(1, 3)
        }
    )]
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(0, 2),
          tile: Coords::new(1, 2),
          anchor: Coords::new(1, 3)
        },
        WallEStepData {
          prev_tile : Coords::new(1, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3)
        }
    )]
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(2, 3)
        },
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(3, 2),
          anchor: Coords::new(3, 1)
        } ; "CCW: Anchor not set to dead-end tile"
    )]
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(3, 2),
          anchor: Coords::new(1, 3)
        },
        WallEStepData {
          prev_tile : Coords::new(3, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(3, 1)
        } ; "CCW: Reset prev tile & update anchor when no next tile (dead-end)"
    )]
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(3, 2),
          tile: Coords::new(2, 2),
          anchor: Coords::new(3, 1)
        },
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(2, 1),
          anchor: Coords::new(2, 0)
        }
    )]
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 2),
          tile: Coords::new(2, 1),
          anchor: Coords::new(2, 0)
        },
        WallEStepData {
          prev_tile : Coords::new(2, 1),
          tile: Coords::new(1, 1),
          anchor: Coords::new(2, 0)
        }
    )]
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(2, 1),
          tile: Coords::new(1, 1),
          anchor: Coords::new(2, 0)
        },
        WallEStepData {
          prev_tile : Coords::new(1, 1),
          tile: Coords::new(1, 0),
          anchor: Coords::new(1, -1)
        }
    )]
    #[test_case(
        WallieDirection::CounterClockwise,
        WallEStepData {
          prev_tile : Coords::new(1, 1),
          tile: Coords::new(1, 0),
          anchor: Coords::new(1, -1)
        },
        WallEStepData {
          prev_tile : Coords::new(1, 0),
          tile: Coords::new(0, 0),
          anchor: Coords::new(-1, 0)
        }
    )]
    #[traced_test]
    fn step(direction: WallieDirection, initial_state: WallEStepData, expected: WallEStepData) {
        let mut grid = TestGrid::from_str(TEST_LVL);
        grid.place_object(
            tile::TileObject {
                entity: Entity::PLACEHOLDER,
                kind: tile::TileObjectKind::Enemy,
            },
            initial_state.tile.into(),
        )
        .expect("Failed to place enemy");
        let mut walle = WallE {
            direction,
            anchor: initial_state.anchor.into(),
            previous_tile: initial_state.prev_tile.into(),
        };
        grid.print_ascii_debug_map(false);

        let next_tile = walle.step(initial_state.tile.into(), &grid);
        let actual = WallEStepData {
            tile: next_tile,
            prev_tile: walle.previous_tile,
            anchor: walle.anchor,
        };

        pretty_assertions::assert_eq!(expected, actual)
    }
}
