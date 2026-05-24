#![allow(unused_imports)]

pub use core::fmt::Debug;
pub use core::hash::Hash;
pub use core::marker::PhantomData;
pub use core::time::Duration;

pub use bevy::audio::Volume;
pub use bevy::diagnostic::FrameCount;
pub use bevy::ecs::entity_disabling::Disabled;
pub use bevy::ecs::spawn::SpawnIter;
pub use bevy::ecs::spawn::SpawnWith;
pub use bevy::input::common_conditions::*;
pub use bevy::math::vec2;
pub use bevy::math::vec3;
pub use bevy::platform::collections::HashMap;
pub use bevy::platform::collections::HashSet;
pub use bevy::prelude::*;
pub use bevy::sprite::Anchor;
pub use bevy::ui::FocusPolicy;
pub use bevy::ui::Val::*;
// pub use bevy_asset_loader::prelude::*;
pub use rand::prelude::*;
pub use rand::rng;
pub use tiny_bail::prelude::*;

pub use mplk_ext::prelude::*;
pub use mplk_tween::prelude::*;
pub use mplk_utils::prelude::*;

pub use crate::UpdateSystems;
pub use crate::gameplay::{
    self, enemy,
    grid::{
        Coords, TileDir, grid, populated, template,
        tile::{self, GridTileCoords, ObjectCoords},
        world::{GridSize, GridWorld},
    },
    input, player,
    text::Text2dWriterExt as _,
    wordlist::{self, char_mask::CharMask, word::Word, wordlist::list::WordList},
};
pub use crate::screens::Screen;
pub use crate::theme::palette::*;
pub use crate::theme::prelude::*;

#[cfg(test)]
pub(crate) use crate::test_utils::TestGrid;
