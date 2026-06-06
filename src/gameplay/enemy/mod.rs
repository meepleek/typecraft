use bevy::prelude::*;

pub mod bouncer;
pub mod wallie;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((wallie::plugin, bouncer::plugin));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Enemy;
