use std::ops::RangeInclusive;

use crate::assets::wordlist::{WordListSource, WordlistAssets};
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnExit(Screen::Loading), update_word_list);
    // todo: also trigger this when some game configuration changes
    // .add_systems(
    //     Update,
    //     update_word_list.run_if(assets_exist.and(resource_changed::<PlayerBindings>)),
    // )
}

#[derive(Resource, Reflect, Debug)]
pub struct WordList {
    pub min_len: usize,
    pub len_indices: Vec<usize>,
    pub words: Vec<Word>,
}
impl WordList {
    pub fn new(source: &WordListSource) -> Self {
        let words = source.words.to_vec();
        let mut len_indices = Vec::with_capacity(20);
        len_indices.push(0);
        let mut curr_len = source.min_len;
        for (i, w) in words.iter().enumerate() {
            if w.len() > curr_len {
                len_indices.push(i);
                curr_len += 1;
            }
        }
        Self {
            min_len: source.min_len,
            len_indices,
            words,
        }
    }

    pub fn iter(&self, len_range: impl Into<RangeInclusive<usize>>) -> impl Iterator<Item = &Word> {
        let range = len_range.into();
        let start = *range.start();
        if start < self.min_len {
            panic!("Invalid lower bound {start}")
        }

        let from = self.len_indices[start - self.min_len];
        let slice = match self.len_indices.get(range.end() - self.min_len + 1) {
            Some(to) => &self.words[from..*to],
            None => &self.words[from..],
        };
        slice.iter()
    }
}

fn update_word_list(
    wordlists: Res<Assets<WordListSource>>,
    wordlist_assets: Res<WordlistAssets>,
    mut cmd: Commands,
) {
    let source = or_return!(wordlists.get(&wordlist_assets.en));
    cmd.insert_resource(WordList::new(source));
}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use pretty_assertions::assert_eq;
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn len_indices() {
        let source = WordListSource {
            min_len: 3,
            words: ["cat", "bat", "hall", "abba", "balls", "basket"]
                .into_iter()
                .map(Word::new)
                .collect(),
        };

        let list = WordList::new(&source);

        assert_eq!(vec![0, 2, 4, 5], list.len_indices);
    }

    #[test_case(3..=3, &["cat", "bat"])]
    #[test_case(3..=4, &["cat", "bat", "hall", "abba"])]
    #[test_case(5..=6, &["balls", "basket"])]
    #[test_case(6..=99, &["basket"])]
    #[traced_test]
    fn iter(range: RangeInclusive<usize>, expected: &[&str]) {
        let source = WordListSource {
            min_len: 3,
            words: ["cat", "bat", "hall", "abba", "balls", "basket"]
                .into_iter()
                .map(Word::new)
                .collect(),
        };

        let list = WordList::new(&source);
        let res = list.iter(range).map(Word::text).collect::<Vec<_>>();

        assert_eq!(expected, res);
    }

    #[test]
    #[should_panic]
    #[traced_test]
    fn iter_low_start_panics() {
        let source = WordListSource {
            min_len: 3,
            words: ["cat", "bat", "hall", "abba", "balls", "basket"]
                .into_iter()
                .map(Word::new)
                .collect(),
        };

        let list = WordList::new(&source);
        _ = list.iter(1..=10);
    }
}
