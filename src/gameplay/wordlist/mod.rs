pub use crate::prelude::*;

pub mod char_mask;
pub mod list;
pub mod word;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(list::plugin);
}
