use crate::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

pub fn wall() -> impl Bundle {
    (Wall, Text2d::new("WALL"), TextFont::from_font_size(30.))
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Wall;
