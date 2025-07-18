#![allow(dead_code)]
use std::collections::HashMap;
use crate::TranslatorService;

pub struct Required;

impl Required {
    pub fn apply<T>(value: &Option<T>) -> bool {
        value.is_some()
    }

    pub fn validate<T>(
        translator_service: &TranslatorService,
        lang: &str,
        value: &Option<T>,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if !Self::apply(value) {
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            v.push(translator_service.variables(&lang, "validation.required", &vars));
        }
        v
    }

    pub fn validated<T, O: FnOnce(&T) -> Vec<String>>(
        translator_service: &TranslatorService,
        lang: &str,
        value: &Option<T>,
        cb: O,
        attribute_name: &str,
    ) -> Vec<String> {
        if Self::apply(value) {
            cb(value.as_ref().unwrap())
        } else {
            let mut v: Vec<String> = Vec::new();
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            v.push(translator_service.variables(&lang, "validation.required", &vars));
            v
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::required::tests::apply
        let value: Option<String> = None;
        assert_eq!(true, Required::apply(&Some("test".to_string())));
        assert_eq!(false, Required::apply(&value));
    }
}
