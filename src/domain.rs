use core::panic;

use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(input: String) -> Self {
        let is_empty_or_whitespace = input.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters // (`a` and `̊`).
        let is_too_long = input.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let has_forbidden_chars = input.chars().any(|c| forbidden_characters.contains(&c));

        if is_empty_or_whitespace || is_too_long || has_forbidden_chars {
            panic!("'{}' is not a valid subscriber name", input);
        } else {
            Self(input)
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
