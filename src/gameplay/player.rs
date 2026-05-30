use crate::{gameplay::grid::template::TemplateTileKind, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(handle_player_hit)
        .add_observer(screenshake_on_hit);
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

fn handle_player_hit(ev: On<PlayerHit>, mut hp: Single<&mut PlayerHp>) {
    hp.0 = hp.saturating_sub(ev.dmg);
    if hp.0 == 0 {
        tracing::warn!("todo: die");
    }
}

fn screenshake_on_hit(_ev: On<PlayerHit>, mut shake: Shakes) {
    shake.add_trauma(0.25);
}
