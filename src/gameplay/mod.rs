use bevy::prelude::*;

mod camera;
pub mod enemy;
pub mod grid;
pub mod input;
pub mod level;
pub mod player;
pub mod text;
pub mod turn;
pub mod wall;
pub mod wordlist;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
        player::plugin,
        grid::plugin,
        input::plugin,
        camera::plugin,
        wall::plugin,
        wordlist::plugin,
        enemy::plugin,
    ));
}
