use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(object_char_typed)
        .add_observer(tween_out_object)
        .add_observer(trigger_fade_in_targetable_char::<ObjectWordsCompleted>)
        .add_observer(trigger_fade_in_targetable_char::<ObjectExploded>)
        .add_observer(remove_exploded_object_from_grid)
        .add_observer(fade_out_exploded_object)
        .add_observer(emit_exploded_object_particles)
        .add_observer(move_object)
        .add_observer(update_wall_word_sections);
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq)]
pub struct ObjectExploded {
    pub entity: Entity,
    pub tile: Coords,
}
impl tile::TileEvent for ObjectExploded {
    fn tile(&self) -> Coords {
        self.tile
    }
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq)]
pub struct ObjectCharTyped(pub Entity);

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq)]
pub struct ObjectWordsCompleted {
    entity: Entity,
    tile: Coords,
}
impl tile::TileEvent for ObjectWordsCompleted {
    fn tile(&self) -> Coords {
        self.tile
    }
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq)]
pub struct ObjectMove {
    pub entity: Entity,
    pub start_tile: Coords,
    pub end_tile: Coords,
}

#[derive(Component, Debug)]
pub struct AllowPlayerCollision {
    pub dmg: u8,
}

fn move_object(
    ev: On<ObjectMove>,
    grid: Option<Single<&mut grid::Grid>>,
    allow_player_collision_q: Query<&AllowPlayerCollision>,
    mut cmd: Commands,
) {
    let mut grid = or_return_quiet!(grid);
    if ev.start_tile == ev.end_tile {
        tracing::warn!(?ev, "invalid tile object move");
        return;
    }
    let e = ev.event_target();
    let world_pos = or_return!(grid.tile_to_world(ev.end_tile));
    let allow_player_collision = allow_player_collision_q.get(e);
    if let Err(err) = grid.move_object(e, ev.end_tile, allow_player_collision.is_ok()) {
        tracing::warn!(?ev, ?err, "Failed to move object");
        return;
    };
    if let Ok(allow_player_collision) = allow_player_collision
        && ev.end_tile == grid.player_tile()
    {
        // todo: these need to be delayed till the object actually moves into the player (+leeway)
        cmd.trigger(player::PlayerHit {
            dmg: allow_player_collision.dmg,
        });
        cmd.trigger(ObjectExploded {
            entity: e,
            tile: ev.end_tile,
        });
    }
    cmd.try_insert_to(
        e,
        TransformPositionLensSrc::new(world_pos).duration(ms(250)),
    );
    cmd.trigger(tile::FadeTargetableTile {
        tile: ev.start_tile,
        direction: FadeDirection::In,
    });
    cmd.trigger(tile::FadeTargetableTile {
        tile: ev.end_tile,
        direction: FadeDirection::Out,
    });
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

// todo: might reset unfinished words on move? but probly in a different system?
fn update_wall_word_sections(
    _ev: On<player::PlayerMoved>,
    grid: Option<Single<&grid::Grid>>,
    mut txt_w: Text2dWriter,
) {
    let grid = or_return_quiet!(grid);
    for (t, wall_e, words) in grid.iter_destroyable_wall_tiles() {
        let active_tile = grid.is_player_ortho_tile(t);
        txt_w.update_tile_text(wall_e, words.text_sections(), active_tile);
    }
}

fn trigger_fade_in_targetable_char<TEv: EntityEvent + tile::TileEvent>(
    ev: On<TEv>,
    mut cmd: Commands,
) {
    let tile = ev.tile();
    cmd.trigger(tile::FadeTargetableTile {
        tile,
        direction: FadeDirection::In,
    });
}

fn remove_exploded_object_from_grid(ev: On<ObjectExploded>, grid: Option<Single<&mut grid::Grid>>) {
    let mut grid = or_return!(grid);
    match grid.clear_object_tile(ev.event_target()) {
        Some(to) => {
            tracing::debug!(removed_tile_object=?to, "cleared exploded tile object");
        }
        None => {
            tracing::warn!("failed to clear exploded object");
        }
    }
}

fn fade_out_exploded_object(ev: On<ObjectExploded>, mut cmd: Commands) {
    cmd.spawn(
        TransformScaleLensSrc::new(Vec2::splat(1.75))
            .duration(ms(200))
            .target(ev.event_target())
            .easing(EaseFunction::QuarticOut)
            .despawn_target_on_completion(),
    );
    cmd.spawn(
        SpriteAlphaLensSrc::new(0.)
            .duration(ms(130))
            .target(ev.event_target())
            .despawn_target_on_completion(),
    );
}

fn emit_exploded_object_particles(_ev: On<ObjectExploded>) {
    // todo:
}
