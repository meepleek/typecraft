use bevy::prelude::*;

pub mod wallie;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((wallie::plugin,));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Enemy;
