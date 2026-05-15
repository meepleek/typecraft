use crate::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

pub fn wall(text: impl Into<String>) -> impl Bundle {
    (Wall, Text2d::new(text), TextFont::from_font_size(30.))
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Wall;
