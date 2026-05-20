use bevy::math::{I16Vec2, U16Vec2};

use crate::{gameplay::wall, prelude::*};
use input::MoveChars;
use template::{GridChunkTemplate, TemplateTileKind};
use tile::TileObjectKind;

#[derive(Debug, PartialEq)]
pub struct PopulatedGrid {
    pub targetable_tiles: HashMap<Coords, char>,
    pub occupied_tiles: HashMap<Coords, tile::TileObjectKind>,
    pub size: U16Vec2,
}
impl PopulatedGrid {
    const DIRS_ORTHO_CONFLICT: &[I16Vec2] = &[
        // diagonals - direct ortho neighbours can actually never conflict when actions are just ortho
        I16Vec2::NEG_ONE,
        I16Vec2::ONE,
        I16Vec2::new(-1, 1),
        I16Vec2::new(1, -1),
        // ortho 2 tiles away
        I16Vec2::new(0, 2),
        I16Vec2::new(2, 0),
        I16Vec2::new(0, -2),
        I16Vec2::new(-2, 0),
    ];

    pub fn new(
        template: GridChunkTemplate,
        wordlist: &WordList,
        move_chars: &MoveChars,
        mut rng: &mut impl Rng,
    ) -> Self {
        let capacity = template.size.element_product() as usize;
        let mut grid = PopulatedGrid {
            targetable_tiles: HashMap::with_capacity(capacity),
            occupied_tiles: HashMap::with_capacity(capacity),
            size: template.size,
        };
        for tt in template.tiles {
            match tt.kind {
                TemplateTileKind::PermaWall => {}
                TemplateTileKind::Empty => {
                    grid.insert_random_targetable_tile(tt.tile, move_chars, &mut rng)
                }
                TemplateTileKind::Wall => grid.add_ititial_wall(tt.tile, wordlist, &mut rng),
                TemplateTileKind::Player => {
                    grid.add_object(tt.tile, TileObjectKind::Player, move_chars, &mut rng)
                }
                TemplateTileKind::Goal => {
                    grid.add_object(tt.tile, TileObjectKind::Goal, move_chars, &mut rng)
                }
            };
        }

        // todo: do a couple of refining wall words iterations
        // grab wall tiles
        // randomise their order
        // then sort by number of words (lowest first)
        // then add/replace words based on any available neighbour words
        // one word at a time to allow other words to also expand their mask beyond the initial word

        grid
    }

    fn add_object(
        &mut self,
        tile: Coords,
        obj_kind: tile::TileObjectKind,
        move_chars: &MoveChars,
        rng: &mut impl Rng,
    ) {
        _ = self.occupied_tiles.insert(tile, obj_kind);
        self.insert_random_targetable_tile(tile, move_chars, rng);
    }

    fn add_ititial_wall(&mut self, tile: Coords, wordlist: &WordList, mut rng: &mut impl Rng) {
        let neighbour_chars = self.ortho_conflict_chars(tile);
        let neighbour_mask = CharMask::deny(neighbour_chars.iter().collect::<String>().bytes());
        let word = wordlist
            .iter(3..=4)
            .filter(|w| w.matches(neighbour_mask))
            .choose(&mut rng)
            .expect("Failed to find a wall word");
        let first_word_mask = word.mask();
        let mut words = wordlist
            .iter(3..=4)
            .filter(|w| *w != word && w.matches(first_word_mask))
            .choose_multiple(&mut rng, 2)
            .iter()
            .map(|w| w.text())
            .collect::<Vec<_>>();
        words.push(word.text());
        words.shuffle(&mut rng);
        _ = self
            .occupied_tiles
            .insert(tile, tile::TileObjectKind::wall(words));
        let targetable_char = word
            .text()
            .chars()
            .choose(&mut rng)
            .expect("Failed to sample targetable char for wall tile");
        self.insert_targetable_tile(tile, targetable_char);
    }

    fn insert_random_targetable_tile(
        &mut self,
        tile: Coords,
        move_chars: &input::MoveChars,
        rng: &mut impl Rng,
    ) {
        let neighbour_chars = self.ortho_conflict_chars(tile);
        let targetable_char = *move_chars
            .iter()
            .filter(|c| !neighbour_chars.contains(*c))
            .choose(rng)
            .expect("Failed to pick random move char");
        self.insert_targetable_tile(tile, targetable_char);
    }

    fn insert_targetable_tile(&mut self, tile: Coords, targetable_char: char) {
        _ = self.targetable_tiles.insert(tile, targetable_char);
    }

    /// Characters of tiles that could conflict when using orthogonal actions
    fn ortho_conflict_chars(&mut self, tile: Coords) -> HashSet<char> {
        // area spanning 2 into each direction to avoid using the same chars for opposite neighbours of a character
        Self::DIRS_ORTHO_CONFLICT
            .iter()
            .flat_map(|dir| {
                let target = tile + dir;
                if *dir == Coords::ZERO {
                    return Vec::new();
                }
                match self.occupied_tiles.get(&target) {
                    Some(kind) => match kind {
                        TileObjectKind::Enemy(word) => word.chars.clone(),
                        TileObjectKind::Wall(words) => {
                            words.words.iter().flat_map(|w| w.chars.clone()).collect()
                        }
                        TileObjectKind::Player | TileObjectKind::Goal => Vec::new(),
                    },
                    None => self
                        .targetable_tiles
                        .get(&target)
                        .map_or_else(|| Vec::new(), |targetable_char| vec![*targetable_char]),
                }
            })
            .collect()
    }

    pub fn spawn<'a>(self, e_cmd: &'a mut EntityCommands<'a>) -> &'a mut EntityCommands<'a> {
        e_cmd.with_children(|b| {
            let mut grid = grid::Grid::new(self.size);
            for (t, c) in self.targetable_tiles {
                let pos = grid.tile_to_world(t).expect(&format!("Invalid cords {t}"));
                let transform_scale0 = Transform::from_translation(pos.extend(0.))
                    .clone()
                    .with_scale(Vec2::ZERO.extend(1.));
                let e = b
                    .spawn((
                        transform_scale0,
                        Text2d::new(c),
                        TextFont::from_font_size(40.),
                        TextColor(Color::WHITE.with_alpha(tile::TILE_ALPHA_HIDDEN)),
                    ))
                    .id();
                grid.targetable_tiles.insert(
                    t,
                    tile::TargetableTile {
                        move_char: c,
                        move_char_e: e,
                    },
                );

                if let Some(kind) = self.occupied_tiles.get(&t) {
                    let entity = match kind {
                        TileObjectKind::Player => {
                            b.spawn((player::player(), transform_scale0.clone())).id()
                        }
                        TileObjectKind::Enemy(_typable_word) => {
                            // todo:
                            b.spawn(()).id()
                        }
                        TileObjectKind::Wall(typable_words) => b
                            .spawn((wall::wall(typable_words), transform_scale0.clone()))
                            .id(),
                        TileObjectKind::Goal => {
                            // todo:
                            b.spawn(()).id()
                        }
                    };

                    grid.place_entity(
                        tile::TileObject {
                            entity,
                            kind: kind.clone(),
                        },
                        t,
                    )
                    .expect("Failed to place tile object");
                }
            }

            b.spawn((grid, Transform::default(), Visibility::default()));
        });
        e_cmd
    }

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

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;
    use crate::assets::wordlist::WordListSource;

    #[test]
    #[traced_test]
    fn insert_targetable_tile() {
        let mut grid = empty_generated_grid();

        grid.insert_targetable_tile(Coords::ONE, 'a');

        pretty_assertions::assert_eq!(HashMap::from([(Coords::ONE, 'a')]), grid.targetable_tiles);
    }

    #[test]
    #[traced_test]
    fn insert_random_targetable_tile() {
        let mut rng = StdRng::seed_from_u64(1);
        let move_chars = MoveChars::default();
        let mut grid = empty_generated_grid();
        grid.insert_random_targetable_tile(Coords::ONE, &move_chars, &mut rng);

        pretty_assertions::assert_eq!(HashMap::from([(Coords::ONE, 's')]), grid.targetable_tiles);
    }

    #[test]
    #[traced_test]
    fn add_object() {
        let mut rng = StdRng::seed_from_u64(1);
        let move_chars = MoveChars::default();
        let mut grid = empty_generated_grid();

        grid.add_object(Coords::ONE, TileObjectKind::Player, &move_chars, &mut rng);

        pretty_assertions::assert_eq!(
            HashMap::from([(Coords::ONE, TileObjectKind::Player)]),
            grid.occupied_tiles
        );
        pretty_assertions::assert_eq!(HashMap::from([(Coords::ONE, 's')]), grid.targetable_tiles);
    }

    #[test]
    #[traced_test]
    fn add_ititial_wall() {
        let mut rng = StdRng::seed_from_u64(42);
        let wordlist = WordList::new(&WordListSource {
            min_len: 3,
            words: ["all", "cat", "rat", "ball", "loong"]
                .into_iter()
                .map(Word::new)
                .collect(),
        });
        let mut grid = empty_generated_grid();

        grid.add_ititial_wall(Coords::ONE, &wordlist, &mut rng);

        pretty_assertions::assert_eq!(
            HashMap::from([(Coords::ONE, TileObjectKind::wall(["rat"]))]),
            grid.occupied_tiles
        );
        pretty_assertions::assert_eq!(HashMap::from([(Coords::ONE, 't')]), grid.targetable_tiles);
    }

    #[test]
    #[traced_test]
    fn new_from_template() {
        let template = TEST_LVL_6X5.parse();
        let mut rng = StdRng::seed_from_u64(42);
        let wordlist = WordList::new(&WordListSource {
            min_len: 3,
            words: TEST_WORDS.iter().copied().map(Word::new).collect(),
        });
        let move_chars = MoveChars::default();

        let grid = PopulatedGrid::new(template.unwrap(), &wordlist, &move_chars, &mut rng);

        pretty_assertions::assert_eq!(U16Vec2::new(6, 5), grid.size);
        assert_tile_hashmap_eq(
            [
                ((3, 0), 'i'),
                ((4, 0), 'j'),
                ((5, 0), 'u'),
                ((0, 1), 's'),
                ((1, 1), 'y'),
                ((2, 1), 'z'),
                ((3, 1), 'e'),
                ((4, 1), 'g'),
                ((5, 1), 'd'),
                ((0, 2), 'w'),
                ((1, 2), 'i'),
                ((2, 2), 'o'),
                ((3, 2), 'm'),
                ((4, 2), 'h'),
                ((5, 2), 'z'),
                ((0, 3), 'u'),
                ((1, 3), 'm'),
                ((2, 3), 't'),
                ((3, 3), 't'),
                ((4, 3), 'y'),
                ((5, 3), 's'),
                ((0, 4), 'y'),
                ((1, 4), 'y'),
                ((2, 4), 'j'),
                ((3, 4), 'f'),
                ((4, 4), 'a'),
                ((5, 4), 'b'),
            ],
            grid.targetable_tiles.clone(),
        );
        assert_tile_hashmap_eq(
            [
                ((3, 0), TileObjectKind::wall(["fit"])),
                ((5, 0), TileObjectKind::wall(["bus"])),
                ((3, 1), TileObjectKind::wall(["ice"])),
                ((4, 1), TileObjectKind::wall(["egg", "leg"])),
                ((5, 1), TileObjectKind::wall(["add", "dad"])),
            ],
            grid.occupied_tiles.clone(),
        );
    }

    #[test_case((0, 4) => vec!['m', 'j', 'w'])]
    #[test_case((3, 2) => vec!['y', 'l', 'z', 'e', 't', 'g', 'i', 'f'])]
    #[traced_test]
    fn ortho_conflict_chars(tile: (i16, i16)) -> Vec<char> {
        let template = TEST_LVL_6X5.parse().unwrap();
        let mut rng = StdRng::seed_from_u64(42);
        let wordlist = WordList::new(&WordListSource {
            min_len: 3,
            words: TEST_WORDS.iter().copied().map(Word::new).collect(),
        });
        let move_chars = MoveChars::default();

        let mut grid = PopulatedGrid::new(template, &wordlist, &move_chars, &mut rng);

        let conflict_chars = grid.ortho_conflict_chars(tile.into());
        conflict_chars.into_iter().collect()
    }

    fn assert_tile_hashmap_eq<T: Debug + PartialEq>(
        expected: impl IntoIterator<Item = (impl Into<Coords>, T)>,
        actual_map: HashMap<Coords, T>,
    ) {
        let mut actual = actual_map.into_iter().collect::<Vec<_>>();
        actual.sort_by_cached_key(|(t, _)| (t.y, t.x));
        let expected = expected
            .into_iter()
            .map(|(t, val)| (t.into(), val))
            .collect::<Vec<_>>();
        pretty_assertions::assert_eq!(expected, actual);
    }

    fn empty_generated_grid() -> PopulatedGrid {
        PopulatedGrid {
            targetable_tiles: HashMap::new(),
            occupied_tiles: HashMap::new(),
            size: U16Vec2::splat(5),
        }
    }

    const TEST_LVL_6X5: &'static str = "
###W.W
...WWW
......
......
......
";

    const TEST_WORDS: &[&str] = &[
        "act", "add", "age", "ago", "aid", "aim", "air", "all", "and", "any", "app", "arm", "art",
        "ask", "bad", "bag", "ban", "bar", "bed", "bee", "beg", "bet", "big", "bin", "bit", "box",
        "boy", "bus", "but", "buy", "bye", "can", "cap", "car", "cat", "cow", "cry", "cup", "cut",
        "dad", "day", "die", "dig", "dog", "dry", "due", "dvd", "ear", "eat", "egg", "end", "eye",
        "fan", "far", "fat", "fee", "few", "fit", "fix", "flu", "fly", "for", "fry", "fun", "fur",
        "gap", "gas", "get", "god", "gun", "guy", "gym", "hat", "her", "hey", "him", "his", "hit",
        "hot", "how", "ice", "ill", "its", "jam", "job", "joy", "key", "kid", "lab", "law", "lay",
        "leg", "let", "lie", "lip", "lot", "low", "mad", "man", "map",
    ];
}
