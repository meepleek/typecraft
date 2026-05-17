use bevy::asset::{AssetLoader, LoadContext, io::Reader};

use super::wordlist::WordListSource;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_asset_loader::<WordListLoader>()
        .init_asset::<WordListSource>();
}

#[derive(Default, TypePath)]
struct WordListLoader;

impl AssetLoader for WordListLoader {
    type Asset = WordListSource;
    type Settings = ();
    type Error = std::io::Error;
    fn extensions(&self) -> &[&str] {
        &["words.txt"]
    }

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        // todo: properly handle UTF8 errors
        let text = String::from_utf8_lossy(&buf);
        let min_len = 3;
        let mut words: Vec<_> = text
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|w| {
                if w.len() < min_len {
                    return false;
                }
                if w.chars()
                    .any(|c| !c.is_ascii_alphabetic() || !c.is_ascii_lowercase())
                {
                    tracing::warn!(word = w, "Invalid chars");
                    return false;
                }

                true
            })
            .map(Word::new)
            .collect();
        words.sort_unstable_by_key(Word::len);
        Ok(WordListSource { min_len, words })
    }
}
