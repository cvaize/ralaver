#![allow(dead_code)]
use std::collections::HashMap;
use crate::TranslatorService;

pub struct Confirmed;

impl Confirmed {
    pub fn apply<T: PartialEq>(a: &T, b: &T) -> bool {
        a.eq(b)
    }

    pub fn validate<T: PartialEq>(
        translator_service: &TranslatorService,
        lang: &str,
        a: &T,
        b: &T,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if !Self::apply(a, b) {
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            v.push(translator_service.variables(&lang, "validation.confirmed", &vars));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::confirmed::tests::apply
        assert_eq!(
            true,
            Confirmed::apply(&"test".to_string(), &"test".to_string())
        );
        assert_eq!(
            false,
            Confirmed::apply(&"test".to_string(), &"test2".to_string())
        );
    }
}
