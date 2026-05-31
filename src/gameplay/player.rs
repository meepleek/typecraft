use crate::{gameplay::grid::template::TemplateTileKind, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(handle_player_hit)
        .add_observer(screenshake_on_hit)
        .add_observer(move_player);
}

#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub struct PlayerMove {
    pub start_tile: Coords,
    pub end_tile: Coords,
}

pub fn player() -> impl Bundle {
    (
        Player,
        PlayerHp(3),
        Text2d::new(TemplateTileKind::PLAYER),
        TextFont::from_font_size(50.),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct PlayerHp(u8);

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerGridState {
    pub tile: Coords,
    pub entity: Entity,
}

#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub struct PlayerHit {
    pub dmg: u8,
}

fn move_player(ev: On<PlayerMove>, grid: Option<Single<&mut grid::Grid>>, mut cmd: Commands) {
    let mut grid = or_return_quiet!(grid);
    if ev.start_tile == ev.end_tile {
        tracing::warn!(?ev, "invalid player move");
        return;
    }
    or_return!(grid.can_place_at(ev.end_tile, false));
    let e = grid.player_state().entity;
    let world_pos = or_return!(grid.tile_to_world(ev.end_tile));
    grid.move_player(ev.end_tile);
    cmd.try_insert_to(
        e,
        TransformPositionLensSrc::new(world_pos).duration(ms(250)),
    );
    // cmd.trigger(tile::FadeTargetableTile {
    //     tile: ev.start_tile,
    //     direction: FadeDirection::In,
    // });
    // cmd.trigger(tile::FadeTargetableTile {
    //     tile: ev.end_tile,
    //     direction: FadeDirection::Out,
    // });
}

fn handle_player_hit(ev: On<PlayerHit>, mut hp: Single<&mut PlayerHp>) {
    hp.0 = hp.saturating_sub(ev.dmg);
    if hp.0 == 0 {
        tracing::warn!("todo: die");
    }
}

fn screenshake_on_hit(_ev: On<PlayerHit>, mut shake: Shakes) {
    shake.add_trauma(0.55);
}
