use unicode_segmentation::UnicodeSegmentation;
#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(input: String) -> Result<Self, String> {
        let is_empty_or_whitespace = input.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters // (`a` and `̊`).
        let is_too_long = input.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let has_forbidden_chars = input.chars().any(|c| forbidden_characters.contains(&c));

        if is_empty_or_whitespace || is_too_long || has_forbidden_chars {
            Err(format!("'{}' is not a valid subscriber name", input))
        } else {
            Ok(Self(input))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_is_valid() {
        let name = "ë".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn long_names_are_rejected() {
        let name = "ë".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_is_rejected() {
        let name = "   ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn name_containing_invalid_char_is_rejected() {
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        for ch in forbidden_characters {
            assert_err!(SubscriberName::parse(ch.to_string()));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Steve Jobs".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
