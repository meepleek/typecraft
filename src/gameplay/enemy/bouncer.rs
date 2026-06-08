use crate::prelude::*;
use bevy::color::palettes::tailwind;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, Bouncer::run_step.run_on_turn_timer());
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BouncerStep {
    Move(Coords),
    Rotate(TileDir),
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Bouncer {
    direction: TileDir,
}
impl Bouncer {
    pub fn new(direction: TileDir) -> Self {
        Self { direction }
    }

    pub fn bundle(direction: TileDir, transform: Transform) -> impl Bundle {
        let bouncer = Self::new(direction);
        (
            super::Enemy,
            object::ContactDamage { dmg: 1 },
            bouncer,
            transform.with_rotation(direction.rotation().to_quat()),
            Text2d::new("^"),
            TextFont::from_font_size(65.),
            TextColor(tailwind::RED_400.with_alpha(1.).into()),
        )
    }

    fn step(&mut self, current_tile: Coords, grid: &grid::Grid) -> BouncerStep {
        use BouncerStep::*;
        use TileDir::*;

        fn valid_target_tile(grid: &grid::Grid, tile: Coords) -> bool {
            !grid.is_wall_tile(tile) && !grid.is_occupied_tile(tile, true)
        }

        let dir = self.direction.direction();
        let target_tile = current_tile + dir;
        match self.direction {
            Ortho(_) => {
                if valid_target_tile(grid, target_tile) {
                    Move(target_tile)
                } else {
                    let reflected_dir = -dir;
                    Rotate(Ortho(
                        TileOrthoDir::from_direction(reflected_dir)
                            .expect("invalid reflect ortho dir"),
                    ))
                }
            }
            Diag(_) => {
                // neighbours in the move dir
                let tile_x = current_tile + dir.with_y(0);
                let tile_y = current_tile + dir.with_x(0);
                let new_dir = match (
                    valid_target_tile(grid, tile_x),
                    valid_target_tile(grid, tile_y),
                    valid_target_tile(grid, target_tile),
                ) {
                    // both neighbours are there or a corner
                    // => reflects back
                    (false, false, _) | (true, true, false) => {
                        let reflected_dir = -dir;
                        Some(reflected_dir)
                    }
                    (true, false, false) => {
                        let reflected_dir = Coords::new(dir.x, -dir.y);
                        Some(reflected_dir)
                    }
                    (false, true, false) => {
                        let reflected_dir = Coords::new(-dir.x, dir.y);
                        Some(reflected_dir)
                    }
                    (true, _, true) | (_, true, true) => None,
                };
                match new_dir {
                    Some(new_dir) => Rotate(Diag(
                        TileDiagDir::from_direction(new_dir).expect("invalid reflect diag dir"),
                    )),
                    None => Move(target_tile),
                }
            }
        }
    }

    fn run_step(
        grid: Option<Single<&grid::Grid>>,
        mut enemy_q: Query<(Entity, &mut Bouncer)>,
        mut cmd: Commands,
    ) {
        let grid = or_return_quiet!(grid);
        for (e, mut bouncer) in &mut enemy_q {
            let start_tile = or_continue!(grid.entity_to_coords(e));
            let step = bouncer.step(start_tile, &grid);
            match step {
                BouncerStep::Move(end_tile) => {
                    cmd.trigger(object::ObjectMove {
                        entity: e,
                        start_tile,
                        end_tile,
                    });
                }
                BouncerStep::Rotate(tile_dir) => {
                    bouncer.direction = tile_dir;
                    cmd.spawn(
                        TransformRotationDegreesLensSrc::new(tile_dir.rotation().as_degrees())
                            .duration(ms(350))
                            .target(e),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;
    use BouncerStep::*;
    use TileDiagDir::*;
    use TileDir::*;
    use TileOrthoDir::*;

    // todo: revamp to use BouncerStep
    #[test_case(
        "
        .WWW
        ..WW
        ....
        ###.
        @G##
        ",
        ((0, 0), Ortho(South)),
        vec![
            Move(Coords::new(0, 1)),
            Move(Coords::new(0, 2)),
            Rotate(Ortho(North)),
            Move(Coords::new(0, 1)),
            Move(Coords::new(0, 0)),
            Rotate(Ortho(South)),
        ] ; "vertical"
    )]
    #[test_case(
        "
        .WWW
        ..WW
        ....
        ###.
        @G##
        ",
        ((0, 1), Ortho(East)),
        vec![
            Move(Coords::new(1, 1)),
            Rotate(Ortho(West)),
            Move(Coords::new(0, 1)),
            Rotate(Ortho(East)),
        ] ; "horizontal"
    )]
    #[test_case(
        "
        ...
        ...
        ...
        ###
        @G#
        ",
        ((1, 0), Diag(NorthEast)),
        vec![
            Rotate(Diag(SouthEast)),
            Move(Coords::new(2, 1)),
            Rotate(Diag(SouthWest)),
            Move(Coords::new(1, 2)),
            Rotate(Diag(NorthWest)),
            Move(Coords::new(0, 1)),
            Rotate(Diag(NorthEast)),
            Move(Coords::new(1, 0)),
        ] ; "diag - SE"
    )]
    #[traced_test]
    fn step(lvl: &str, initial_state: ((i16, i16), TileDir), expected_steps: Vec<BouncerStep>) {
        if expected_steps.len() < 2 {
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
        let mut bouncer = Bouncer {
            direction: initial_state.1,
        };

        for expected_step in expected_steps {
            let actual_step = bouncer.step(tile, &grid);

            if expected_step != actual_step {
                tracing::warn!(?expected_step, ?actual_step, ?bouncer, ?tile);
                grid.print_ascii_debug_map(false);
            }

            pretty_assertions::assert_eq!(expected_step, actual_step);

            match actual_step {
                Move(next_tile) => {
                    grid.move_object(entity, next_tile, true)
                        .expect("Failed to move enemy");
                    tile = next_tile;
                }
                Rotate(next_dir) => bouncer.direction = next_dir,
            }
        }
    }
}
