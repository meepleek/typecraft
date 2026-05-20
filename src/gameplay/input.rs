use bevy::input::{ButtonState, keyboard::KeyboardInput};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<MoveChars>();
    app.add_systems(Update, handle_input);
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct MoveChars(HashSet<char>);
impl Default for MoveChars {
    fn default() -> Self {
        // todo: default to regular QWERTY instead, but make this configurable
        Self("zarstdhneiokjwfpgcluybvm".chars().collect::<HashSet<_>>())
    }
}

fn handle_input(
    mut input_r: MessageReader<KeyboardInput>,
    grid: Option<Single<&grid::Grid>>,
    mut cmd: Commands,
) {
    let grid = or_return_quiet!(grid);
    let player_state = grid.player_state();
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
            .targetable_neighbours(player_state.tile, tile::TileDirection::Orthogonal)
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
        match &tn.object {
            Some(_to) => todo!(),
            None => {
                cmd.try_insert_to(player_state.entity, tile::ObjectCoords(tn.tile));
            }
        }
    }
}
