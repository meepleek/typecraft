#![allow(dead_code)]

use crate::prelude::*;

pub struct Word {
    word: String,
    mask: u32,
}
impl Word {
    pub fn new(word: impl Into<String>) -> Self {
        let word = word.into();
        Self {
            mask: Self::word_mask(&word),
            word,
        }
    }

    pub fn matches_mask(&self, mask: u32) -> bool {
        self.mask & !mask == 0
    }

    pub fn word_mask(word: &str) -> u32 {
        let mut mask = 0;
        for char_byte in word.bytes() {
            let bit = char_byte - b'a';
            mask |= 1 << bit;
        }
        mask
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;

    #[test_case("a" => 0b1)]
    #[test_case("c" => 0b100)]
    #[test_case("cab" => 0b111)]
    #[test_case("abba" => 0b11)]
    #[test_case("bad" => 0b1011)]
    #[test_case("bid" => 0b100001010)]
    #[traced_test]
    fn word_mask(word: &str) -> u32 {
        Word::word_mask(word)
    }

    #[test_case("a", 0b1 => true)]
    #[test_case("a", 0b111 => true)]
    #[test_case("a", 0b110 => false; "'a' not in mask")]
    #[test_case("cab", 0b11010111 => true)]
    #[test_case("cab", 0b11010101 => false ; "'b' not in mask")]
    #[test_case("abba", 0b11 => true)]
    #[traced_test]
    fn matches_mask(word: &str, mask: u32) -> bool {
        let word = Word::new(word);
        word.matches_mask(mask)
    }
}
