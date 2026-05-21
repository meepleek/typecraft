use crate::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

pub fn wall(words: &tile::TypableWords, active: bool) -> impl Bundle {
    (
        Wall,
        Text2d::new(""),
        TextFont::from_font_size(30.),
        words.child_sections(active),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Wall;
