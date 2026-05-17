use bevy_asset_loader::prelude::*;

use crate::prelude::*;

mod word_loader;
pub mod wordlist;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(word_loader::plugin);
    app.add_loading_state(
        LoadingState::new(Screen::Loading)
            .continue_to_state(if cfg!(feature = "dev") {
                Screen::Gameplay
            } else {
                // Screen::MainMenu
                Screen::Gameplay
            })
            // .load_collection::<SpriteAssets>()
            // .load_collection::<FontAssets>()
            // .load_collection::<SfxAssets>()
            // .load_collection::<MusicAssets>()
            .load_collection::<wordlist::WordlistAssets>(),
    );
    // app.add_systems(Startup, setup_particles);
}
