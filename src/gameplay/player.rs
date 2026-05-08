//! Player-specific behavior.

use bevy::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

pub fn player() -> impl Bundle {
    (Text2d::new("@"), TextFont::from_font_size(50.))
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player;
