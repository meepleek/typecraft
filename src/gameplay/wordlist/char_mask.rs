use crate::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy, Deref, DerefMut, Reflect)]
pub struct CharMask(u32);
impl CharMask {
    // mask bits for a-z
    const LETTER_MASK: u32 = (1 << (b'z' - b'a' + 1)) - 1;

    pub fn allow(char_bytes: impl Iterator<Item = u8>) -> Self {
        Self(Self::mask(char_bytes, true))
    }

    pub fn deny(char_bytes: impl Iterator<Item = u8>) -> Self {
        Self(Self::mask(char_bytes, false))
    }

    pub fn matches(self, other: CharMask) -> bool {
        self.0 & !other.0 == 0
    }

    fn mask(char_bytes: impl Iterator<Item = u8>, allow: bool) -> u32 {
        let mut mask = 0;
        for char_byte in char_bytes {
            let bit = char_byte - b'a';
            mask |= 1 << bit;
        }
        if !allow {
            // invert mask to deny the provided char bits
            mask = !mask;
            // discard bits above 'z'
            mask &= Self::LETTER_MASK;
        }
        mask
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tracing_test::traced_test;

    use super::*;

    #[test]
    fn letter_mask() {
        assert_eq!(CharMask::LETTER_MASK, 0b11111111111111111111111111)
    }

    #[test_case("a" => 0b1)]
    #[test_case("c" => 0b100)]
    #[test_case("cab" => 0b111)]
    #[test_case("abba" => 0b11)]
    #[test_case("bad" => 0b1011)]
    #[test_case("bid" => 0b100001010)]
    #[traced_test]
    fn allow_mask(word: &str) -> u32 {
        CharMask::allow(word.bytes()).0
    }

    #[test_case("a", "a" => true)]
    #[test_case("a", "abc" => true)]
    #[test_case("a", "bc" => false; "'a' not in mask")]
    #[test_case("cab", "abcmn" => true)]
    #[test_case("cab", "acmnz" => false ; "'b' not in mask")]
    #[test_case("abba", "ab" => true)]
    #[traced_test]
    fn matches_mask(word: &str, mask_str: &str) -> bool {
        let word_mask = CharMask::allow(word.bytes());
        word_mask.matches(CharMask::allow(mask_str.bytes()))
    }

    #[test_case("a" => 0b11111111111111111111111110)]
    #[test_case("c" => 0b11111111111111111111111011)]
    #[test_case("cab" => 0b11111111111111111111111000)]
    #[test_case("abba" => 0b11111111111111111111111100)]
    #[test_case("bad" => 0b11111111111111111111110100)]
    #[test_case("bid" => 0b11111111111111111011110101)]
    #[traced_test]
    fn deny_mask(word: &str) -> u32 {
        CharMask::deny(word.bytes()).0
    }

    #[test_case("a", "a" => false)]
    #[test_case("a", "abc" => false)]
    #[test_case("a", "bc" => true)]
    #[test_case("cab", "b" => false)]
    #[test_case("cab", "mnz" => true)]
    #[test_case("abba", "a" => false)]
    #[test_case("abba", "cd" => true)]
    #[traced_test]
    fn matches_deny_mask(word: &str, mask_str: &str) -> bool {
        let word_mask = CharMask::allow(word.bytes());
        word_mask.matches(CharMask::deny(mask_str.bytes()))
    }
}
