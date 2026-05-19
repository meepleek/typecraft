use std::str::FromStr;

use crate::prelude::*;
use bevy::math::U16Vec2;

#[derive(Debug, PartialEq)]
pub struct TemplateTile {
    pub tile: Coords,
    pub kind: TemplateTileKind,
}

#[derive(Debug, PartialEq)]
pub enum TemplateTileKind {
    PermaWall,
    Empty,
    Wall,
    Player,
    Goal,
    // Enemy,
}
impl TemplateTileKind {
    pub const PERMAWALL: char = '#';
    pub const EMPTY: char = '.';
    pub const WALL: char = 'W';
    pub const PLAYER: char = '@';
    pub const GOAL: char = 'G';
    // pub const ENEMY: char = '#';
}

impl TryFrom<char> for TemplateTileKind {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            Self::PERMAWALL => Ok(Self::PermaWall),
            Self::EMPTY => Ok(Self::Empty),
            Self::WALL => Ok(Self::Wall),
            Self::PLAYER => Ok(Self::Player),
            Self::GOAL => Ok(Self::Goal),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GridChunkTemplate {
    pub size: U16Vec2,
    pub tiles: Vec<TemplateTile>,
    pub player: Option<Coords>,
    pub goal: Option<Coords>,
}
impl GridChunkTemplate {
    // pub fn spawn<'a>(
    //     self,
    //     e_cmd: &'a mut EntityCommands<'a>,
    //     wordlist: &WordList,
    // ) -> &'a mut EntityCommands<'a> {
    //     e_cmd.with_children(|b| {
    //         let mut rng = rand::rng();
    //         let mut grid = grid::Grid::new(self.size.x, self.size.y);
    //         for tt in self.tiles {
    //             let pos = grid
    //                 .tile_to_world(tt.tile)
    //                 .expect(&format!("Invalid cords {}", tt.tile));
    //             let transform = Transform::from_translation(pos.extend(0.));
    //             let transform_scale0 = transform.clone().with_scale(Vec2::ZERO.extend(1.));
    //             let spawn_targetable_char = match tt.kind {
    //                 TemplateTileKind::PermaWall => None,
    //                 TemplateTileKind::Empty => Some((None, true)),
    //                 TemplateTileKind::Wall => {
    //                     let neighbour_chars = grid.ortho_conflict_chars(tt.tile);
    //                     let neighbour_mask =
    //                         CharMask::deny(neighbour_chars.iter().collect::<String>().bytes());
    //                     // todo: use a char from the first word as the move char of the tile
    //                     let word = wordlist
    //                         .iter(3..=4)
    //                         .filter(|w| w.matches(neighbour_mask))
    //                         .choose(&mut rng)
    //                         .expect("Failed to find a wall word");
    //                     let first_word_mask = word.mask();
    //                     let mut words = wordlist
    //                         .iter(3..=4)
    //                         .filter(|w| *w != word && w.matches(first_word_mask))
    //                         .choose_multiple(&mut rng, 2)
    //                         .iter()
    //                         .map(|w| w.text())
    //                         .collect::<Vec<_>>();
    //                     words.push(word.text());
    //                     words.shuffle(&mut rng);
    //                     tracing::warn!(?words);

    //                     let e = b
    //                         .spawn((wall::wall(word.text()), transform_scale0.clone()))
    //                         .id();
    //                     grid.place_entity(
    //                         tile::TileObject {
    //                             entity: e,
    //                             kind: tile::TileObjectKind::wall(words),
    //                         },
    //                         tt.tile,
    //                     )
    //                     .expect("Failed to place tile object");

    //                     Some((Some(e), false))
    //                 }
    //                 TemplateTileKind::Player => {
    //                     let e = b.spawn((player::player(), transform_scale0.clone())).id();
    //                     grid.place_entity(
    //                         tile::TileObject {
    //                             entity: e,
    //                             kind: tile::TileObjectKind::Player,
    //                         },
    //                         tt.tile,
    //                     )
    //                     .expect("Failed to place tile object");

    //                     Some((Some(e), false))
    //                 }
    //                 TemplateTileKind::Goal => {
    //                     // todo: goal
    //                     Some((None, true))
    //                 }
    //             };
    //             if let Some((tween_e_src, show_char)) = spawn_targetable_char {
    //                 const TWEEN_STEP_MS: u64 = 110;
    //                 let chess_dist_to_player = self.player.chebyshev_distance(tt.tile);
    //                 let manhattan_dist_to_player = self.player.manhattan_distance(tt.tile);
    //                 let tween_delay = ms(chess_dist_to_player as u64 * TWEEN_STEP_MS);
    //                 let neighbour_chars = grid.ortho_conflict_chars(tt.tile);

    //                 let mut targetable_char_e = None;
    //                 for _ in 0..100 {
    //                     let c = grid
    //                         .move_chars
    //                         .iter()
    //                         .choose(&mut rng)
    //                         .expect("Failed to pick random move char");
    //                     if !neighbour_chars.contains(c) {
    //                         let (t, start_alpha) = if show_char {
    //                             (transform_scale0, tile::TILE_ALPHA_TARGETABLE)
    //                         } else {
    //                             (transform, tile::TILE_ALPHA_HIDDEN)
    //                         };
    //                         let e = b
    //                             .spawn((
    //                                 t,
    //                                 Text2d::new(*c),
    //                                 TextFont::from_font_size(40.),
    //                                 TextColor(Color::WHITE.with_alpha(start_alpha)),
    //                                 tile::CharWiggle::new(
    //                                     ms(rng.random_range(0..5_000)),
    //                                     chess_dist_to_player,
    //                                 ),
    //                             ))
    //                             .id();
    //                         if show_char && manhattan_dist_to_player > 1 {
    //                             b.spawn(
    //                                 TextAlphaLensSrc::absolute(
    //                                     start_alpha,
    //                                     tile::TILE_ALPHA_INACTIVE,
    //                                 )
    //                                 .duration(ms(210))
    //                                 .delay(tween_delay + ms(150))
    //                                 .target(e),
    //                             );
    //                         }

    //                         grid.targetable_tiles.insert(
    //                             tt.tile,
    //                             tile::TargetableTile {
    //                                 move_char: *c,
    //                                 move_char_e: e,
    //                             },
    //                         );
    //                         targetable_char_e = Some(e);
    //                         break;
    //                     }
    //                 }

    //                 let tween_e = tween_e_src
    //                     .unwrap_or(targetable_char_e.expect("Failed to spawn targetable tile"));
    //                 b.spawn(
    //                     TransformScaleLensSrc::new(Vec2::ONE)
    //                         .duration(ms(400))
    //                         .target(tween_e)
    //                         .delay(tween_delay)
    //                         .easing(EaseFunction::BackOut),
    //                 );
    //             }
    //         }

    //         // todo: do a couple of refining wall words iterations
    //         // grab wall tiles
    //         // randomise their order
    //         // then sort by number of words (lowest first)
    //         // then add/replace words based on any available neighbour words
    //         // one word at a time to allow other words to also expand their mask beyond the initial word

    //         b.spawn((grid, Transform::default(), Visibility::default()));
    //     });
    //     e_cmd
    // }
}

impl FromStr for GridChunkTemplate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut size = Coords::ZERO;
        let mut errors = Vec::new();
        let mut tiles = Vec::with_capacity(s.len());
        let mut player = Vec::with_capacity(1);
        let mut goal = Vec::with_capacity(1);

        for (tile, c) in s
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .map(move |(x, c)| (Coords::new(x as i16, y as i16), c))
            })
        {
            size = size.max(tile);
            match TemplateTileKind::try_from(c) {
                Ok(kind) => {
                    match kind {
                        TemplateTileKind::Player => player.push(tile),
                        TemplateTileKind::Goal => goal.push(tile),
                        _ => {}
                    }

                    tiles.push(TemplateTile { tile, kind });
                }
                Err(_) => {
                    errors.push(format!("Invalid char '{c}' at {tile}"));
                }
            }
        }

        if player.len() > 1 {
            errors.push("Multiple players".to_string());
        }
        if goal.len() > 1 {
            errors.push("Multiple goals".to_string());
        }

        if errors.is_empty() {
            Ok(GridChunkTemplate {
                size: (size + Coords::ONE).as_u16vec2(),
                tiles,
                player: player.pop(),
                goal: goal.pop(),
            })
        } else {
            Err(errors.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;
    use TemplateTileKind::*;

    #[test]
    #[traced_test]
    fn parse_ok() {
        let lvl = "
##..WW
#@..WG
";
        let parsed_template = lvl.parse::<GridChunkTemplate>();

        assert_eq!(
            Ok(GridChunkTemplate {
                size: U16Vec2::new(6, 2),
                tiles: [
                    ((0, 0), PermaWall),
                    ((1, 0), PermaWall),
                    ((2, 0), Empty),
                    ((3, 0), Empty),
                    ((4, 0), Wall),
                    ((5, 0), Wall),
                    ((0, 1), PermaWall),
                    ((1, 1), Player),
                    ((2, 1), Empty),
                    ((3, 1), Empty),
                    ((4, 1), Wall),
                    ((5, 1), Goal),
                ]
                .into_iter()
                .map(|(tile, kind)| TemplateTile {
                    tile: tile.into(),
                    kind
                })
                .collect(),
                player: Some(Coords::new(1, 1)),
                goal: Some(Coords::new(5, 1)),
            }),
            parsed_template
        );
    }

    #[test]
    #[traced_test]
    fn parse_ok_6x5() {
        let lvl = "
###WGW
...WWW
......
.@....
......
";
        let template = lvl.parse::<GridChunkTemplate>();
        pretty_assertions::assert_eq!(
            Ok(GridChunkTemplate {
                size: U16Vec2::new(6, 5),
                tiles: [
                    ((0, 0), TemplateTileKind::PermaWall),
                    ((1, 0), TemplateTileKind::PermaWall),
                    ((2, 0), TemplateTileKind::PermaWall),
                    ((3, 0), TemplateTileKind::Wall),
                    ((4, 0), TemplateTileKind::Goal),
                    ((5, 0), TemplateTileKind::Wall),
                    ((0, 1), TemplateTileKind::Empty),
                    ((1, 1), TemplateTileKind::Empty),
                    ((2, 1), TemplateTileKind::Empty),
                    ((3, 1), TemplateTileKind::Wall),
                    ((4, 1), TemplateTileKind::Wall),
                    ((5, 1), TemplateTileKind::Wall),
                    ((0, 2), TemplateTileKind::Empty),
                    ((1, 2), TemplateTileKind::Empty),
                    ((2, 2), TemplateTileKind::Empty),
                    ((3, 2), TemplateTileKind::Empty),
                    ((4, 2), TemplateTileKind::Empty),
                    ((5, 2), TemplateTileKind::Empty),
                    ((0, 3), TemplateTileKind::Empty),
                    ((1, 3), TemplateTileKind::Player),
                    ((2, 3), TemplateTileKind::Empty),
                    ((3, 3), TemplateTileKind::Empty),
                    ((4, 3), TemplateTileKind::Empty),
                    ((5, 3), TemplateTileKind::Empty),
                    ((0, 4), TemplateTileKind::Empty),
                    ((1, 4), TemplateTileKind::Empty),
                    ((2, 4), TemplateTileKind::Empty),
                    ((3, 4), TemplateTileKind::Empty),
                    ((4, 4), TemplateTileKind::Empty),
                    ((5, 4), TemplateTileKind::Empty),
                ]
                .into_iter()
                .map(|(tile, kind)| template::TemplateTile {
                    tile: tile.into(),
                    kind
                })
                .collect(),
                player: Some(Coords::new(1, 3)),
                goal: Some(Coords::new(4, 0)),
            }),
            template,
        );
    }

    #[test]
    #[traced_test]
    fn parse_ok_whitespace_trimmed() {
        let lvl = " \t @.G  ";
        let parsed_template = lvl.parse::<GridChunkTemplate>();

        assert_eq!(
            Ok(GridChunkTemplate {
                size: U16Vec2::new(3, 1),
                tiles: [((0, 0), Player), ((1, 0), Empty), ((2, 0), Goal),]
                    .into_iter()
                    .map(|(tile, kind)| TemplateTile {
                        tile: tile.into(),
                        kind
                    })
                    .collect(),
                player: Some(Coords::new(0, 0)),
                goal: Some(Coords::new(2, 0)),
            }),
            parsed_template
        );
    }

    #[test]
    #[traced_test]
    fn parse_ok_no_playr_or_goal() {
        let lvl = ".";
        let parsed_template = lvl.parse::<GridChunkTemplate>();

        assert_eq!(
            Ok(GridChunkTemplate {
                size: U16Vec2::new(1, 1),
                tiles: vec![TemplateTile {
                    tile: Coords::ZERO,
                    kind: Empty
                }],
                player: None,
                goal: None,
            }),
            parsed_template
        );
    }

    #[test_case("@@.G", "Multiple players")]
    #[test_case("@.GG", "Multiple goals")]
    #[test_case("@.$._.G", "Invalid char '$' at [2, 0]\nInvalid char '_' at [4, 0]")]
    #[traced_test]
    fn parse_missing_objects(lvl: &str, err_msg: &str) {
        let parsed_template = lvl.parse::<GridChunkTemplate>();

        assert_eq!(Err(err_msg.to_string()), parsed_template);
    }
}
