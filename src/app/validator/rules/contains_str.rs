#![allow(dead_code)]
use std::collections::HashMap;
use crate::TranslatorService;

pub struct ContainsStr;

impl ContainsStr {
    pub fn apply(value: &str, contains: &[&str]) -> bool {
        contains.contains(&value)
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &str,
        contains: &[&str],
        attribute_name: &str,
    ) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if !Self::apply(value, contains) {
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            v.push(translator_service.variables(&lang, "validation.in", &vars));
        }
        v
    }

    pub fn validated<O: FnOnce(&str) -> Vec<String>>(
        translator_service: &TranslatorService,
        lang: &str,
        value: &str,
        contains: &[&str],
        cb: O,
        attribute_name: &str,
    ) -> Vec<String> {
        if Self::apply(value, contains) {
            cb(value)
        } else {
            let mut v: Vec<String> = Vec::new();
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            v.push(translator_service.variables(&lang, "validation.in", &vars));
            v
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::contains::tests::apply
        assert!(ContainsStr::apply("test", &["test", "test2"]));
        assert!(!ContainsStr::apply("test3", &["test", "test2"]));
    }
}
