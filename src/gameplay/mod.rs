use bevy::prelude::*;

mod camera;
pub mod grid;
pub mod input;
pub mod level;
pub mod player;
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
    ));
}
