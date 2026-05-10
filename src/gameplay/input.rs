use bevy::input::{ButtonState, keyboard::KeyboardInput};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, handle_input);
}

fn handle_input(
    mut input_r: MessageReader<KeyboardInput>,
    grid: Option<Single<&mut grid::Grid>>,
    mut cmd: Commands,
) {
    let mut grid = or_return_quiet!(grid);
    let (player_tile, player_e) = or_return!(grid.get_player());
    let input_chars = input_r
        .read()
        .filter_map(|ev| {
            if !ev.repeat && ev.state == ButtonState::Pressed {
                ev.text.as_ref().and_then(|txt| {
                    let chars: Vec<_> = txt.chars().collect();
                    if chars.len() == 1 {
                        Some(*chars.first().unwrap())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for c in input_chars {
        let targetable_neighbours = grid
            .targetable_neighbours(player_tile, tile::TileDirection::All)
            .filter(|tn| tn.next_char().is_some_and(|tile_c| tile_c == c))
            .collect::<Vec<_>>();
        if targetable_neighbours.len() > 1 {
            tracing::warn!(
                ?c,
                ?targetable_neighbours,
                "There are multiple target tiles for this char"
            );
        }
        let tn = or_continue_quiet!(targetable_neighbours.first());
        tracing::warn!(?tn);
        match &tn.object {
            Some(_to) => todo!(),
            None => {
                // todo: just update the TileCoords for the entity
                // then upgate the grid & tween from a different system
                // also need to fade in/out the from/to move chars

                let world_pos = or_return!(grid.tile_to_world(tn.tile));
                or_return!(grid.move_entity(player_e, tn.tile));

                cmd.try_insert_to(
                    player_e,
                    mplk_tween::prelude::TransformPositionLensSrc::new(world_pos).duration(ms(250)),
                );
            }
        }
    }
}
