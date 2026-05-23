use bevy::{color::palettes::tailwind, time::common_conditions::on_real_timer};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // app.add_systems(Update, wall_e_move.run_if(on_real_timer(ms(1500))));
    app.add_systems(Update, wall_e_move.run_if(on_real_timer(ms(500))));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct WallE {
    anchor: Coords,
    previous_tile: Coords,
}
impl WallE {
    fn step(&mut self, current_tile: Coords, grid: &grid::Grid) -> Coords {
        let next_tile = match grid
            .unoccupied_targetable_neighbours(current_tile, tile::TileDirection::Orthogonal)
            .filter(|utn| {
                utn.tile != self.previous_tile && utn.tile.chebyshev_distance(self.anchor) == 1
            })
            .next()
        {
            Some(next_tile) => {
                let next_tile = next_tile.tile;
                // updating anchor
                let dir = next_tile - current_tile;
                // handle corner anchors
                let turning_around_anchor = next_tile.manhattan_distance(self.anchor) == 1;
                let maybe_corner_tile = next_tile + dir;
                let moved_anchor_tile = self.anchor + dir;
                if grid.is_wall_tile(maybe_corner_tile)
                    // prevent setting a dead-end anchor
                    && grid
                        .unoccupied_targetable_neighbours(
                            maybe_corner_tile,
                            tile::TileDirection::All,
                        )
                        .any(|utn| utn.tile != next_tile)
                {
                    self.anchor = maybe_corner_tile;
                } else if !turning_around_anchor
                    && next_tile.manhattan_distance(moved_anchor_tile) == 1
                    && grid.is_wall_tile(moved_anchor_tile)
                {
                    self.anchor = moved_anchor_tile;
                }
                self.previous_tile = current_tile;
                next_tile
            }
            None => {
                // no matching tile => return back
                let next_tile = self.previous_tile;
                self.previous_tile = current_tile;
                next_tile
            }
        };
        next_tile
    }
}

pub fn wall_e(tile: Coords, anchor: Coords) -> impl Bundle {
    (
        super::Enemy,
        ObjectCoords(tile),
        WallE {
            anchor,
            // used as a deny list for movement
            // so using the initial position is fine
            previous_tile: tile,
        },
        Text2d::new(template::TemplateTileKind::ENEMY),
        TextFont::from_font_size(50.),
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
    fn step(initial_state: WallEStepData, expected: WallEStepData) {
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
