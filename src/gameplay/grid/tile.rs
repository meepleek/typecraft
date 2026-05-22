use bevy::{
    color::palettes::tailwind, ecs::relationship::RelatedSpawner, math::U16Vec2, text::TextBounds,
};
use mplk_utils::math::asymptotic_smoothing_with_delta_time;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (move_tile_object,).in_set(UpdateSystems::Grid),
            (
                fade_in_tile,
                update_wall_word_sections,
                tween_targetable_chars_alpha_on_player_move,
                update_tile_char_wiggle_target_speed,
                wiggle_tile_char,
            )
                .after(UpdateSystems::Visuals),
        ),
    );

    // todo: fixme - this is here just to sort out ordering issues with some systems even when using labels
    for sys in [
        UpdateSystems::TickTimers,
        UpdateSystems::RecordInput,
        UpdateSystems::Grid,
        UpdateSystems::Visuals,
    ] {
        let system = move |mut _cmd: Commands| {};
        app.add_systems(Update, system.in_set(sys).run_if(run_once));
    }
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

    pub fn advance(&mut self) -> bool {
        self.completed_count += 1;
        self.completed()
    }

    fn empty_word_text_spans() -> [ObjectTextSpan; 4] {
        default()
    }

    fn active_word_text_spans(&self, trailing_nl: bool) -> [ObjectTextSpan; 4] {
        let mut spans = Self::empty_word_text_spans();
        // already written
        if self.completed_count > 0 {
            spans[0] = ObjectTextSpan {
                text: self.chars[..self.completed_count].iter().collect(),
                style: ObjectTextStyle::Written,
            };
        }
        // caret
        spans[1] = ObjectTextSpan {
            text: "|".to_string(),
            style: ObjectTextStyle::Caret,
        };
        let completed = self.completed();
        // active char
        if !completed {
            spans[2] = ObjectTextSpan {
                text: self.chars[self.completed_count].to_string(),
                style: ObjectTextStyle::ActiveChar,
            };
        }
        // pending chars
        spans[3] = ObjectTextSpan {
            text: Self::word_text(&self.chars[self.completed_count + 1..], trailing_nl),
            style: ObjectTextStyle::PendingChars,
        };
        spans
    }

    fn upcoming_word_text_spans(&self, trailing_nl: bool) -> [ObjectTextSpan; 4] {
        let mut spans = Self::empty_word_text_spans();
        spans[1] = ObjectTextSpan {
            text: " ".to_string(),
            style: ObjectTextStyle::Empty,
        };
        spans[3] = ObjectTextSpan {
            text: Self::word_text(&self.chars[..], trailing_nl),
            style: ObjectTextStyle::UpcomingWord,
        };
        spans
    }

    fn completed_word_text_spans(&self, trailing_nl: bool) -> [ObjectTextSpan; 4] {
        let mut spans = Self::empty_word_text_spans();
        spans[0] = ObjectTextSpan {
            text: Self::word_text(&self.chars[..], trailing_nl),
            style: ObjectTextStyle::Written,
        };
        spans
    }

    fn word_text(chars: &[char], trailing_nl: bool) -> String {
        let mut text: String = chars.iter().collect();
        if trailing_nl {
            text.push('\n');
        }
        text
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

    pub fn completed(&self) -> bool {
        self.completed_word_count == self.words.len()
    }

    pub fn next_char(&self) -> Option<char> {
        self.words
            .get(self.completed_word_count)
            .and_then(TypableWord::next_char)
    }

    pub fn advance(&mut self) -> bool {
        if let Some(w) = self.words.get_mut(self.completed_word_count)
            && w.advance()
        {
            self.completed_word_count += 1;
            return self.completed();
        }

        false
    }

    pub fn text_sections(&self) -> Vec<ObjectTextSpan> {
        self.words
            .iter()
            .enumerate()
            .flat_map(|(i, w)| {
                let trailing_nl = i < self.words.len() - 1;
                // already completed word
                if i < self.completed_word_count {
                    return w.completed_word_text_spans(trailing_nl);
                }
                // current word
                if i == self.completed_word_count {
                    w.active_word_text_spans(trailing_nl)
                }
                // upcoming word
                else {
                    w.upcoming_word_text_spans(trailing_nl)
                }
            })
            .collect()
    }

    pub fn child_sections(&self, active: bool) -> impl Bundle {
        let sections = self
            .text_sections()
            .into_iter()
            .map(|section| section.span(active))
            .collect::<Vec<_>>();
        Children::spawn(SpawnWith(move |s: &mut RelatedSpawner<_>| {
            for section in sections {
                s.spawn(section);
            }
        }))
    }
}

#[derive(Default)]
pub struct ObjectTextSpan {
    pub text: String,
    pub style: ObjectTextStyle,
}
impl ObjectTextSpan {
    pub fn span(&self, active: bool) -> impl Bundle + use<> {
        (
            TextSpan::new(&self.text),
            TextColor(self.style.colour(active)),
        )
    }
}

#[derive(Default)]
pub enum ObjectTextStyle {
    #[default]
    Empty,
    Written,
    ActiveChar,
    Caret,
    PendingChars,
    UpcomingWord,
}
impl ObjectTextStyle {
    pub fn colour(&self, active: bool) -> Color {
        match (self, active) {
            (ObjectTextStyle::Written, _) => tailwind::GRAY_500,
            (ObjectTextStyle::ActiveChar | ObjectTextStyle::Caret, true) => tailwind::GREEN_400,
            (ObjectTextStyle::PendingChars, true) => tailwind::GRAY_200,
            (
                ObjectTextStyle::PendingChars
                | ObjectTextStyle::ActiveChar
                | ObjectTextStyle::Caret,
                false,
            )
            | (ObjectTextStyle::UpcomingWord, _) => tailwind::GRAY_400,
            (ObjectTextStyle::Empty, _) => Srgba::default().with_alpha(0.),
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
    tile_q: Populated<(Entity, &ObjectCoords, Has<player::Player>), Changed<ObjectCoords>>,
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
            or_return!(grid.move_object(player_e, tile))
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

fn fade_in_tile(
    tile_q: Query<(Entity, &GridTileCoords), Added<GridTileCoords>>,
    grid: Option<Single<&mut grid::Grid>>,
    mut trans_q: Query<&mut Transform>,
    mut cmd: Commands,
) {
    let grid = or_return!(grid);
    const TWEEN_STEP_MS: u64 = 110;
    for (e, t) in &tile_q {
        let t = t.0;
        let tween_delay = ms(grid.chess_distance_to_player(t) as u64 * TWEEN_STEP_MS);
        let tween_e = match grid.get_tile_object_or_player_entity(t) {
            Some(obj_e) => {
                // reset scale of hidden targetable char
                let mut char_t = or_return!(trans_q.get_mut(e));
                char_t.scale = Vec3::ONE;
                obj_e
            }
            None => {
                let alpha = if grid.is_player_ortho_tile(t) {
                    tile::TILE_ALPHA_TARGETABLE
                } else {
                    tile::TILE_ALPHA_INACTIVE
                };
                cmd.spawn(
                    TextAlphaLensSrc::new(alpha)
                        .duration(ms(210))
                        .delay(tween_delay + ms(80))
                        .target(e),
                );
                e
            }
        };

        cmd.spawn(
            TransformScaleLensSrc::new(Vec2::ONE)
                .duration(ms(550))
                .target(tween_e)
                .delay(tween_delay)
                .easing(EaseFunction::BackOut),
        );
    }
}

fn tween_targetable_chars_alpha_on_player_move(
    _: Populated<(), (With<player::Player>, Changed<ObjectCoords>)>,
    grid: Option<Single<&grid::Grid>>,
    mut cmd: Commands,
) {
    let grid = or_return_quiet!(grid);
    for (t, tt) in grid.iter_movable_tiles(true) {
        cmd.spawn(
            TextAlphaLensSrc::new(grid.targetable_char_alpha(t))
                .duration(grid::Grid::TARGETABLE_TILE_FADE)
                .target(tt.move_char_e),
        );
    }
}

// todo: move to wall
fn update_wall_word_sections(
    _: Populated<(), (With<player::Player>, Changed<ObjectCoords>)>,
    grid: Option<Single<&mut grid::Grid>>,
    mut txt_w: Text2dWriter,
) {
    let grid = or_return_quiet!(grid);
    for (t, wall_e, words) in grid.iter_wall_tiles() {
        let active_tile = grid.is_player_ortho_tile(t);
        txt_w.update_tile_text(wall_e, words.text_sections(), active_tile);
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
    _: Populated<&ObjectCoords, (With<player::Player>, Changed<ObjectCoords>)>,
    grid: Option<Single<&mut grid::Grid>>,
    mut wiggle_q: Query<&mut CharWiggle>,
) {
    let grid = or_return_quiet!(grid);
    for (t, tt) in grid.iter_targetable_tiles() {
        let mut char_rotation = or_continue!(wiggle_q.get_mut(tt.move_char_e));
        char_rotation.update_target_speed(grid.chess_distance_to_player(t));
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
