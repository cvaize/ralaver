#![allow(dead_code)]
use crate::app::validator::rules::str_max_chars_count::StrMaxCharsCount;
use crate::app::validator::rules::str_min_chars_count::StrMinCharsCount;
use crate::TranslatorService;

pub struct StrMinMaxCharsCount;

impl StrMinMaxCharsCount {
    pub fn apply(value: &str, min: usize, max: usize) -> bool {
        StrMinCharsCount::apply(value, min) && StrMaxCharsCount::apply(value, max)
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
            StrMinCharsCount::validate(translator_service, lang, value, min, attribute_name);
        let mut errors2 =
            StrMaxCharsCount::validate(translator_service, lang, value, max, attribute_name);

        errors.append(&mut errors2);
        errors
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::str_max_min_chars_count::tests::apply
        let s = "汤ДАЙЁ_35YuLx76xar";
        let len = s.len();
        let chars_count = s.chars().count();
        assert_ne!(len, chars_count);

        assert!(StrMinMaxCharsCount::apply(s, chars_count - 2, chars_count + 2));
        assert!(!StrMinMaxCharsCount::apply(s, chars_count + 2, chars_count + 2));
    }
}
