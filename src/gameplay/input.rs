use bevy::input::{ButtonState, keyboard::KeyboardInput};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (handle_input, move_tile_object));
}

fn handle_input(
    mut input_r: MessageReader<KeyboardInput>,
    grid: Option<Single<&grid::Grid>>,
    mut cmd: Commands,
) {
    let grid = or_return_quiet!(grid);
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
                cmd.try_insert_to(player_e, tile::TileCoords(tn.tile));
            }
        }
    }
}

fn move_tile_object(
    tile_q: Query<(Entity, &TileCoords), Changed<TileCoords>>,
    grid: Option<Single<&mut grid::Grid>>,
    mut cmd: Commands,
) {
    let mut grid = or_return_quiet!(grid);
    let (_, player_e) = or_return!(grid.get_player());
    for (e, tc) in tile_q {
        let tile = tc.0;
        // also need to fade in/out the from/to move chars

        let world_pos = or_return!(grid.tile_to_world(tile));
        let (start_tile, end_tile) = or_return!(grid.move_entity(player_e, tile));
        // todo: actually the alpha should tween a different entity containing the move char...
        cmd.try_insert_to(
            e,
            TransformPositionLensSrc::new(world_pos).duration(ms(250)),
        );
        cmd.try_insert_to(
            start_tile.move_char_e,
            TextAlphaLensSrc::new(1.).duration(ms(150)),
        );
        cmd.try_insert_to(
            end_tile.move_char_e,
            TextAlphaLensSrc::new(0.).duration(ms(150)),
        );
    }
}
