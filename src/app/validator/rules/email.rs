#![allow(dead_code)]
use std::collections::HashMap;
use crate::TranslatorService;

pub struct Email;

impl Email {
    pub fn apply(value: &str) -> bool {
        value.len() >= 3 && value.len() <= 255 && value.contains("@")
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &str,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if !Self::apply(value) {
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            v.push(translator_service.variables(&lang, "validation.email", &vars));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::email::tests::apply
        assert_eq!(true, Email::apply(&"test@test".to_string()));
        assert_eq!(false, Email::apply(&"test".to_string()));
    }
}
