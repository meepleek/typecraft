use bevy::color::palettes::tailwind;
use bevy_firefly::prelude::Occluder2d;

use crate::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

pub fn wall(words: &tile::TypableWords, active: bool) -> impl Bundle {
    let occluder_half_size = 96. / 2.;
    (
        Wall,
        Text2d::new(""),
        TextFont::from_font_size(30.),
        // TextBounds::new_horizontal(96.),
        // TextLayout::new_with_justify(Justify::Left),
        words.child_sections(active),
        // Occluder2d::polygon_cc([
        //     occluder_half_size * TileDir::NORTH_EAST.as_vec2(),
        //     occluder_half_size * TileDir::SOUTH_EAST.as_vec2(),
        //     occluder_half_size * TileDir::SOUTH_WEST.as_vec2(),
        //     occluder_half_size * TileDir::NORTH_WEST.as_vec2(),
        // ])
        // .expect("failed to create polygon occluder")
        // // .with_opacity(1.)
        // .with_z_sorting(false),
        Occluder2d::rectangle(96., 96.),
        // .with_z_sorting(false)
        // .with_color(tailwind::AMBER_600.into()),
        Sprite::from_color(tailwind::AMBER_800, Vec2::splat(96.)),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Wall;
