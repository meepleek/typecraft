#![allow(dead_code)]

use crate::prelude::*;
use wordlist::char_mask::CharMask;

pub struct Word {
    word: String,
    mask: CharMask,
}
impl Word {
    pub fn new(word: impl Into<String>) -> Self {
        let word = word.into();
        Self {
            mask: CharMask::allow(word.bytes()),
            word,
        }
    }
}
