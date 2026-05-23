use bevy::{color::palettes::tailwind, time::common_conditions::on_real_timer};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, wall_e_move.run_if(on_real_timer(ms(1500))));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct WallE {
    anchor: Coords,
    previous_tile: Coords,
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
        let current_tile = coords.0;
        let next_tile = match grid
            .unoccupied_targetable_neighbours(coords.0, tile::TileDirection::All)
            .filter(|utn| {
                utn.tile != walle.previous_tile && utn.tile.chebyshev_distance(walle.anchor) == 1
            })
            .next()
        {
            Some(next_tile) => {
                let next_tile = next_tile.tile;
                // updating anchor
                let dir = next_tile - current_tile;
                // handle corner anchors
                let turning_around_anchor = next_tile.manhattan_distance(walle.anchor) == 1;
                let maybe_corner_tile = next_tile + dir;
                let moved_anchor_tile = walle.anchor + dir;
                if grid.is_wall_tile(maybe_corner_tile) {
                    walle.anchor = maybe_corner_tile;
                } else if !turning_around_anchor
                    && next_tile.manhattan_distance(moved_anchor_tile) == 1
                    && grid.is_wall_tile(moved_anchor_tile)
                {
                    walle.anchor = moved_anchor_tile;
                }
                next_tile
            }
            None => {
                // no matching tile => return back
                let next_tile = walle.previous_tile;
                walle.previous_tile = current_tile;
                next_tile
            }
        };
        coords.0 = next_tile;
    }
}
