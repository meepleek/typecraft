#![allow(dead_code)]

use bevy::{color::palettes::tailwind, ecs::relationship::RelatedSpawner, math::U16Vec2};
use mplk_utils::math::asymptotic_smoothing_with_delta_time;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            move_tile_object,
            fade_in_tile,
            tween_tile_char_alpha,
            update_tile_char_wiggle_target_speed,
            wiggle_tile_char,
        ),
    );
}

pub const TILE_ALPHA_INACTIVE: f32 = 0.15;
pub const TILE_ALPHA_TARGETABLE: f32 = 1.0;
pub const TILE_ALPHA_HIDDEN: f32 = 0.0;

#[derive(Component, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct ObjectCoords(pub Coords);

#[derive(Component, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct GridTileCoords(pub Coords);

#[derive(Debug, Clone)]
pub struct TargetableTile {
    pub move_char: char,
    pub move_char_e: Entity,
}

// use this as a single source of truth for both the movement & ability direction
// to avoid tricky combos like ortho movement + diag attack that could lead to buggy pathfinding
// this should also simplify the UI & mental overhead for players
#[derive(Component, Debug, Clone, Copy)]
pub enum TileDirection {
    Orthogonal,
    Diagonal,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileObject {
    pub entity: Entity,
    pub kind: TileObjectKind,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum TileObjectKind {
    Enemy(TypableWord),
    Wall(TypableWords),
    Goal,
}
impl TileObjectKind {
    pub fn enemy(word: impl Into<String>) -> Self {
        TileObjectKind::Enemy(TypableWord::new(word.into().chars().collect::<Vec<_>>()))
    }

    pub fn wall<TText: Into<String>>(words: impl IntoIterator<Item = TText>) -> Self {
        TileObjectKind::Wall(TypableWords::new(words))
    }

    pub fn next_char(&self) -> Option<char> {
        match self {
            Self::Goal => None,
            Self::Enemy(word) => word.next_char(),
            Self::Wall(words) => words.next_char(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypableWord {
    pub chars: Vec<char>,
    completed_count: usize,
}
impl TypableWord {
    pub fn new(chars: impl Into<Vec<char>>) -> Self {
        Self {
            chars: chars.into(),
            completed_count: 0,
        }
    }

    pub fn next_char(&self) -> Option<char> {
        self.chars.get(self.completed_count).copied()
    }

    pub fn completed(&self) -> bool {
        self.completed_count == self.chars.len()
    }

    pub fn active_word_text_sections(&self) -> Vec<ObjectTextSpan> {
        let mut sections = Vec::with_capacity(3);
        if self.completed_count > 0 {
            sections.push(ObjectTextSpan {
                text: self.chars[..self.completed_count].iter().collect(),
                style: ObjectTextStyle::Written,
            });
        }
        let completed = self.completed();
        if !completed {
            sections.push(ObjectTextSpan {
                text: format!("{}|", self.chars[self.completed_count]),
                style: ObjectTextStyle::Active,
            });
        }
        if completed {
            sections.push(ObjectTextSpan {
                text: "|".to_string(),
                style: ObjectTextStyle::Written,
            });
        } else {
            sections.push(ObjectTextSpan {
                text: self.chars[(self.completed_count + 1)..].iter().collect(),
                style: ObjectTextStyle::Pending,
            });
        }

        sections
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypableWords {
    pub words: Vec<TypableWord>,
    completed_word_count: usize,
}
impl TypableWords {
    pub fn new<TText: Into<String>>(words: impl IntoIterator<Item = TText>) -> Self {
        Self {
            completed_word_count: 0,
            words: words
                .into_iter()
                .map(|w| TypableWord::new(w.into().chars().collect::<Vec<_>>()))
                .collect(),
        }
    }

    pub fn next_char(&self) -> Option<char> {
        self.words
            .get(self.completed_word_count)
            .and_then(TypableWord::next_char)
    }

    pub fn text_sections(&self) -> Vec<ObjectTextSpan> {
        self.words
            .iter()
            .enumerate()
            .flat_map(|(i, w)| {
                // already completed word
                if i <= self.completed_word_count {
                    return Vec::default();
                }
                // current word
                if i == self.completed_word_count + 1 {
                    w.active_word_text_sections()
                }
                // upcoming word
                else {
                    let mut text: String = w.chars.iter().copied().collect();
                    if i == self.completed_word_count + 2 {
                        // follows active word
                        text.insert(0, '\n');
                    }
                    if i < self.words.len() - 1 {
                        // more upcoming words
                        text.push('\n');
                    }
                    vec![ObjectTextSpan {
                        text,
                        style: ObjectTextStyle::Upcoming,
                    }]
                }
            })
            .collect()
    }

    pub fn child_sections(&self) -> impl Bundle {
        let sections = self
            .text_sections()
            .into_iter()
            .map(|section| section.span())
            .collect::<Vec<_>>();
        Children::spawn(SpawnWith(move |s: &mut RelatedSpawner<_>| {
            for section in sections {
                s.spawn(section);
            }
        }))
    }
}

pub struct ObjectTextSpan {
    text: String,
    style: ObjectTextStyle,
}
impl ObjectTextSpan {
    pub fn span(&self) -> impl Bundle + use<> {
        (TextSpan::new(&self.text), TextColor(self.style.colour()))
    }
}

pub enum ObjectTextStyle {
    Written,
    Active,
    Pending,
    Upcoming,
}
impl ObjectTextStyle {
    pub fn colour(&self) -> Color {
        match self {
            ObjectTextStyle::Written => tailwind::GRAY_700,
            ObjectTextStyle::Active => tailwind::GREEN_400,
            ObjectTextStyle::Pending => tailwind::GRAY_100,
            ObjectTextStyle::Upcoming => tailwind::GRAY_300,
        }
        .into()
    }
}

#[derive(Debug)]
pub struct TargetableNeighbour {
    pub tile: Coords,
    pub targetable: TargetableTile,
    pub object: Option<TileObject>,
}
impl TargetableNeighbour {
    pub fn next_char(&self) -> Option<char> {
        match &self.object {
            Some(to) => to.kind.next_char(),
            None => Some(self.targetable.move_char),
        }
    }
}

pub struct TileIterator {
    grid_size: U16Vec2,
    tile: Coords,
    start_tile: Coords,
}
impl Iterator for TileIterator {
    type Item = Coords;

    fn next(&mut self) -> Option<Self::Item> {
        let size = self.grid_size.as_i16vec2();
        if self.tile.y - self.start_tile.y >= size.y as i16 {
            None
        } else {
            let next = self.tile;
            self.tile.x += 1;
            if self.tile.x - self.start_tile.x == size.x {
                self.tile = (self.start_tile.x, self.tile.y + 1).into();
            }
            Some(next)
        }
    }
}
impl TileIterator {
    pub fn from_size(grid_size: impl Into<U16Vec2>) -> Self {
        Self::new(Coords::ZERO, grid_size)
    }

    pub fn centered(grid_size: impl Into<U16Vec2>) -> Self {
        let grid_size = grid_size.into();
        Self::new(-grid_size.as_i16vec2() / 2, grid_size)
    }

    fn new(centre: impl Into<Coords>, grid_size: impl Into<U16Vec2>) -> Self {
        let tile = centre.into();
        Self {
            grid_size: grid_size.into(),
            tile,
            start_tile: tile,
        }
    }
}

fn move_tile_object(
    tile_q: Query<(Entity, &ObjectCoords, Has<player::Player>), Changed<ObjectCoords>>,
    grid: Option<Single<&mut grid::Grid>>,
    mut cmd: Commands,
) {
    let mut grid = or_return_quiet!(grid);
    let player_e = grid.player_state().entity;
    for (e, tc, is_player) in tile_q {
        let tile = tc.0;
        // also need to fade in/out the from/to move chars
        let world_pos = or_return!(grid.tile_to_world(tile));
        let (start_tile, end_tile) = if is_player {
            let start_tile = or_return!(grid.targetable_tiles.get(&grid.player_tile())).clone();
            let end_tile = or_return!(grid.targetable_tiles.get(&tile)).clone();
            grid.move_player(tile);
            (start_tile, end_tile)
        } else {
            or_return!(grid.move_entity(player_e, tile))
        };
        cmd.try_insert_to(
            e,
            TransformPositionLensSrc::new(world_pos).duration(ms(250)),
        );
        cmd.try_insert_to(
            start_tile.move_char_e,
            TextAlphaLensSrc::new(1.).duration(ms(150)),
        );
        cmd.try_insert_to(
            end_tile.move_char_e,
            TextAlphaLensSrc::new(0.).duration(ms(150)),
        );
    }
}

fn fade_in_tile() {
    // const TWEEN_STEP_MS: u64 = 110;
    // let chess_dist_to_player = self.player.chebyshev_distance(t);
    // let manhattan_dist_to_player = self.player.manhattan_distance(tt.tile);
    // let tween_delay = ms(chess_dist_to_player as u64 * TWEEN_STEP_MS);

    // tile::CharWiggle::new(ms(rng.random_range(0..5_000)), chess_dist_to_player);

    // if show_char && manhattan_dist_to_player > 1 {
    //     b.spawn(
    //         TextAlphaLensSrc::absolute(start_alpha, tile::TILE_ALPHA_INACTIVE)
    //             .duration(ms(210))
    //             .delay(tween_delay + ms(150))
    //             .target(e),
    //     );
    // }

    // let tween_e =
    //     tween_e_src.unwrap_or(targetable_char_e.expect("Failed to spawn targetable tile"));
    // b.spawn(
    //     TransformScaleLensSrc::new(Vec2::ONE)
    //         .duration(ms(400))
    //         .target(tween_e)
    //         .delay(tween_delay)
    //         .easing(EaseFunction::BackOut),
    // );
}

fn tween_tile_char_alpha(
    player_q: Option<Single<&ObjectCoords, (With<player::Player>, Changed<ObjectCoords>)>>,
    grid: Option<Single<&mut grid::Grid>>,
    mut cmd: Commands,
) {
    let grid = or_return_quiet!(grid);
    let player_t = or_return_quiet!(player_q).0;
    for (t, tt) in grid.iter_movable_tiles(true) {
        let mut alpha = TILE_ALPHA_INACTIVE;
        let dist_manhattan = player_t.manhattan_distance(t);
        if t == player_t {
            alpha = TILE_ALPHA_HIDDEN;
        } else if dist_manhattan == 1 {
            alpha = TILE_ALPHA_TARGETABLE;
        }
        cmd.try_insert_to(
            tt.move_char_e,
            TextAlphaLensSrc::new(alpha).duration(ms(150)),
        );
    }
}

#[derive(Component, Debug)]
pub struct CharWiggle {
    offset: Duration,
    target_speed: f32,
}
impl CharWiggle {
    pub fn new(anim_offset: Duration, chess_distance: u16) -> Self {
        let speed = Self::target_speed(chess_distance);
        Self {
            offset: anim_offset,
            target_speed: speed,
        }
    }

    fn update_target_speed(&mut self, chess_distance: u16) {
        self.target_speed = Self::target_speed(chess_distance);
    }

    fn target_speed(chess_distance: u16) -> f32 {
        if chess_distance <= 1 { 2.25 } else { 1.15 }
    }
}

fn update_tile_char_wiggle_target_speed(
    player: Option<Single<&ObjectCoords, (With<player::Player>, Changed<ObjectCoords>)>>,
    grid: Option<Single<&mut grid::Grid>>,
    mut wiggle_q: Query<&mut CharWiggle>,
) {
    let grid = or_return_quiet!(grid);
    let player_t = or_return_quiet!(player).0;
    for (t, tt) in grid.iter_targetable_tiles() {
        let mut char_rotation = or_continue!(wiggle_q.get_mut(tt.move_char_e));
        char_rotation.update_target_speed(player_t.chebyshev_distance(t));
    }
}

fn wiggle_tile_char(mut rotation_q: Query<(&CharWiggle, &mut Transform)>, time: Res<Time>) {
    for (rot, mut t) in &mut rotation_q {
        let max_deg: f32 = 15.;
        let mult = ((time.elapsed_secs() + rot.offset.as_secs_f32()) * rot.target_speed).sin();
        let target_deg = max_deg * mult;
        // todo: this can jump a lil bit, so ideally just fix that the rotation should continue from the same point, but just the speed should be updated
        let rot_deg = asymptotic_smoothing_with_delta_time(
            t.rotation.to_rot2().as_degrees(),
            target_deg,
            0.25,
            time.delta_secs(),
        );
        t.rotation = Quat::from_rotation_z(rot_deg.to_radians());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_size() {
        let tiles: Vec<_> = TileIterator::from_size((5, 3)).collect();
        assert_eq!(
            tiles,
            [
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (4, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
                (4, 2),
            ]
            .map(Into::into)
        );
    }

    #[test]
    fn centered() {
        let tiles: Vec<_> = TileIterator::centered((5, 3)).collect();
        assert_eq!(
            tiles,
            [
                (-2, -1),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (-2, 1),
                (-1, 1),
                (0, 1),
                (1, 1),
                (2, 1),
            ]
            .map(Into::into)
        );
    }
}
