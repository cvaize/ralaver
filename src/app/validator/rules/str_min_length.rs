#![allow(dead_code)]
use crate::TranslatorService;
use std::collections::HashMap;

pub struct StrMinLength;

impl StrMinLength {
    pub fn apply(value: &str, min: usize) -> bool {
        value.len() >= min
    }

    //noinspection DuplicatedCode
    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &str,
        min: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if !Self::apply(value, min) {
            let m = min.to_string();
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            vars.insert("min", m.as_str());
            v.push(translator_service.variables(&lang, "validation.min.string", &vars));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::str_min_length::tests::apply
        let s = "汤ДАЙЁ_35YuLx76xar";
        let len = s.len();
        let chars_count = s.chars().count();
        assert_ne!(len, chars_count);

        assert!(StrMinLength::apply(s, len - 2));
        assert!(!StrMinLength::apply(s, len + 2));
    }
}
