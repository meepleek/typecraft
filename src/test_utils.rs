use crate::{assets::wordlist::WordListSource, prelude::*};
use grid::Grid;
use populated::PopulatedGrid;
use template::GridChunkTemplate;
use tile::{TargetableTile, TileObject};

#[cfg(test)]
use ansi_term::Color;

#[cfg(test)]
pub enum DebugGridTileColor {
    Header,
    White,
    Red,
    Green,
    Dimmed,
}
#[cfg(test)]
impl DebugGridTileColor {
    pub const BG_COL: Color = Color::RGB(40, 40, 40);

    pub fn colored(&self, tile_char: char) -> String {
        self.style().paint(tile_char.to_string()).to_string()
    }

    pub fn prefix(&self) -> String {
        self.style().prefix().to_string()
    }

    fn style(&self) -> ansi_term::Style {
        (match self {
            DebugGridTileColor::Header => Color::RGB(170, 170, 170),
            DebugGridTileColor::White => Color::RGB(255, 255, 255),
            DebugGridTileColor::Red => Color::Red,
            DebugGridTileColor::Green => Color::Green,
            DebugGridTileColor::Dimmed => Color::RGB(170, 170, 170),
        })
        .on(Self::BG_COL)
    }
}

#[cfg(test)]
pub(crate) struct TestGrid {}
#[cfg(test)]
impl TestGrid {
    pub const TILE_SIZE: u16 = 96;

    pub const WORDS: &[&str] = &[
        "act", "add", "age", "ago", "aid", "aim", "air", "all", "and", "any", "app", "arm", "art",
        "ask", "bad", "bag", "ban", "bar", "bed", "bee", "beg", "bet", "big", "bin", "bit", "box",
        "boy", "bus", "but", "buy", "bye", "can", "cap", "car", "cat", "cow", "cry", "cup", "cut",
        "dad", "day", "die", "dig", "dog", "dry", "due", "dvd", "ear", "eat", "egg", "end", "eye",
        "fan", "far", "fat", "fee", "few", "fit", "fix", "flu", "fly", "for", "fry", "fun", "fur",
        "gap", "gas", "get", "god", "gun", "guy", "gym", "hat", "her", "hey", "him", "his", "hit",
        "hot", "how", "ice", "ill", "its", "jam", "job", "joy", "key", "kid", "lab", "law", "lay",
        "leg", "let", "lie", "lip", "lot", "low", "mad", "man", "map",
    ];

    pub fn from_populated(populated: PopulatedGrid) -> Grid {
        let mut e_idx = 0;
        let mut get_e = || {
            e_idx += 1;
            Entity::from_raw_u32(e_idx).unwrap()
        };
        let mut grid = Grid::new(
            populated.grid_size,
            populated.tile_size,
            player::PlayerGridState {
                tile: populated.player_tile,
                entity: get_e(),
            },
        );
        grid.set_targetable_tiles(
            populated
                .targetable_tiles
                .into_iter()
                .map(|(t, c)| {
                    (
                        t,
                        TargetableTile {
                            move_char: c,
                            move_char_e: get_e(),
                        },
                    )
                })
                .collect(),
        );
        for (t, kind) in populated.occupied_tiles {
            grid.place_object(
                TileObject {
                    entity: get_e(),
                    kind: kind,
                },
                t,
            )
            .expect("Failed to place test grid tile object");
        }
        grid
    }

    pub fn from_str(lvl_str: &str) -> Grid {
        Self::from_str_with_rng(lvl_str, &mut Self::seeded_rng())
    }

    pub fn from_str_with_rng(lvl_str: &str, rng: &mut impl Rng) -> Grid {
        let template: GridChunkTemplate = lvl_str.parse().expect("Failed to parse test lvl");
        let wordlist = WordList::new(&WordListSource {
            min_len: 3,
            words: Self::WORDS.iter().copied().map(Word::new).collect(),
        });
        let move_chars = input::MoveChars::default();
        let populated =
            populated::PopulatedGrid::new(Self::TILE_SIZE, template, &wordlist, &move_chars, rng);
        Self::from_populated(populated)
    }

    pub fn rng_from_seed(seed: u64) -> impl Rng {
        StdRng::seed_from_u64(seed.into())
    }

    pub fn seeded_rng() -> impl Rng {
        Self::rng_from_seed(42)
    }
}
