use core::range::RangeInclusive;

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
    min_len: usize,
    len_indices: Vec<usize>,
    words: Vec<Word>,
}
impl WordList {
    pub fn iter(&self, len_range: impl Into<RangeInclusive<usize>>) -> impl Iterator<Item = &Word> {
        let range = len_range.into();
        let from = self.len_indices[range.start - self.min_len];
        let to = self.len_indices[range.last - self.min_len + 1];
        self.words[from..=to].iter()
    }
}

fn update_word_list(
    wordlists: Res<Assets<WordListSource>>,
    wordlist_assets: Res<WordlistAssets>,
    mut cmd: Commands,
) {
    let source = or_return!(wordlists.get(&wordlist_assets.en));

    let words = source.words.to_vec();
    let mut len_indices = Vec::with_capacity(20);
    len_indices.push(0);
    let mut curr_len = source.min_len;
    for (i, w) in words.iter().enumerate() {
        tracing::warn!(?w);
        if w.len() > curr_len {
            len_indices.push(i);
            curr_len += 1;
        }
    }

    cmd.insert_resource(WordList {
        min_len: source.min_len,
        len_indices,
        words,
    });
}
