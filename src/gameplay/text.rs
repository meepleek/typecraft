use crate::prelude::*;

pub trait Text2dWriterExt {
    fn update_tile_text(
        &mut self,
        object_e: Entity,
        spans: impl IntoIterator<Item = tile::ObjectTextSpan>,
        active_tile: bool,
    );
}
impl<'w, 's> Text2dWriterExt for Text2dWriter<'w, 's> {
    fn update_tile_text(
        &mut self,
        object_e: Entity,
        spans: impl IntoIterator<Item = tile::ObjectTextSpan>,
        active_tile: bool,
    ) {
        for (i, section) in spans.into_iter().enumerate() {
            let i = i + 1; // include root text index
            let col = section.style.colour(active_tile);
            *self.text(object_e, i) = section.text;
            *self.color(object_e, i) = col.into();
        }
    }
}
