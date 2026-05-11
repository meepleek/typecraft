use crate::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

pub fn player() -> impl Bundle {
    (Player, Text2d::new("@"), TextFont::from_font_size(50.))
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;
