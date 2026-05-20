use crate::{gameplay::grid::template::TemplateTileKind, prelude::*};

pub(super) fn plugin(_app: &mut App) {}

pub fn player() -> impl Bundle {
    (
        Player,
        Text2d::new(TemplateTileKind::PLAYER),
        TextFont::from_font_size(50.),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerGridState {
    pub tile: Coords,
    pub entity: Entity,
}
