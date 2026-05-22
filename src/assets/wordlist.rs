use crate::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Asset, TypePath, Debug)]
pub struct WordListSource {
    pub min_len: usize,
    pub words: Vec<Word>,
}

#[derive(AssetCollection, Resource)]
pub struct WordlistAssets {
    #[asset(path = "en.words.txt")]
    pub en: Handle<WordListSource>,
}
