use bevy::prelude::*;

pub mod wall_e;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((wall_e::plugin,));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Enemy;
