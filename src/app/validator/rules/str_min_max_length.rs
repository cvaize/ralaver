#![allow(dead_code)]

use crate::app::validator::rules::str_max_length::StrMaxLength;
use crate::app::validator::rules::str_min_length::StrMinLength;
use crate::TranslatorService;

pub struct StrMinMaxLength;

impl StrMinMaxLength {
    pub fn apply(value: &str, min: usize, max: usize) -> bool {
        StrMinLength::apply(value, min) && StrMaxLength::apply(value, max)
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &str,
        min: usize,
        max: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut errors =
            StrMinLength::validate(translator_service, lang, value, min, attribute_name);
        let mut errors2 =
            StrMaxLength::validate(translator_service, lang, value, max, attribute_name);

        errors.append(&mut errors2);
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::str_max_min_length::tests::apply
        let s = "汤ДАЙЁ_35YuLx76xar";
        let len = s.len();
        let chars_count = s.chars().count();
        assert_ne!(len, chars_count);

        assert!(StrMinMaxLength::apply(s, len - 2, len + 2));
        assert!(!StrMinMaxLength::apply(s, len + 2, len + 2));
    }
}
