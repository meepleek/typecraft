use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(object_char_typed)
        .add_observer(tween_out_object)
        .add_observer(fade_in_targetable_word);
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq)]
pub struct ObjectCharTyped(pub Entity);

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq)]
pub struct ObjectWordsCompleted {
    entity: Entity,
    tile: Coords,
}

fn object_char_typed(
    ev: On<ObjectCharTyped>,
    grid: Option<Single<&mut grid::Grid>>,
    mut txt_w: Text2dWriter,
    mut cmd: Commands,
) {
    let mut grid = or_return!(grid);
    let e = ev.event_target();
    let tile = or_return!(grid.entity_to_coords(e));
    let active_tile = grid.is_player_ortho_tile(tile);
    let obj = or_return!(grid.get_object_mut(tile));
    let Some(typable_words) = obj.kind.words_mut() else {
        return;
    };
    let completed = typable_words.advance();
    txt_w.update_tile_text(e, typable_words.text_sections(), active_tile);
    if completed {
        tracing::debug!(?tile, "clearing object words tile");
        grid.clear_tile(tile);
        cmd.trigger(ObjectWordsCompleted { entity: e, tile });
    }
}

fn tween_out_object(ev: On<ObjectWordsCompleted>, mut cmd: Commands) {
    cmd.spawn(
        TransformScaleLensSrc::new(Vec2::ZERO)
            .duration(ms(300))
            .easing(EaseFunction::BackIn)
            .target(ev.event_target())
            .despawn_target_on_completion(),
    );
}

fn fade_in_targetable_word(
    ev: On<ObjectWordsCompleted>,
    grid: Option<Single<&grid::Grid>>,
    mut cmd: Commands,
) {
    let grid = or_return!(grid);
    let e = or_return!(grid.get_targetable_tile(ev.tile).map(|tt| tt.move_char_e));
    cmd.spawn(
        TextAlphaLensSrc::new(grid.targetable_char_alpha(ev.tile))
            .duration(grid::Grid::TARGETABLE_TILE_FADE)
            .target(e),
    );
}
