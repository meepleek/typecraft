use crate::prelude::*;
use wordlist::char_mask::CharMask;

#[derive(Reflect, Debug, Clone)]
pub struct Word {
    word: String,
    mask: CharMask,
    len: usize,
}
impl Word {
    pub fn new(word: impl Into<String>) -> Self {
        let word = word.into();
        Self {
            len: word.len(),
            mask: CharMask::allow(word.bytes()),
            word,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn matches(&self, mask: CharMask) -> bool {
        self.mask.matches(mask)
    }

    pub fn text(&self) -> String {
        self.word.clone()
    }
}
