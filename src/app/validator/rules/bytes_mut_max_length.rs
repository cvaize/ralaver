#![allow(dead_code)]
use crate::TranslatorService;
use std::collections::HashMap;
use bytes::BytesMut;

pub struct BytesMutMaxLength;

impl BytesMutMaxLength {
    pub fn apply(value: &BytesMut, max: usize) -> bool {
        value.len() <= max
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &BytesMut,
        max: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if !Self::apply(value, max) {
            let m = max.to_string();
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            vars.insert("max", m.as_str());
            v.push(translator_service.variables(&lang, "validation.max.file", &vars));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use super::*;

    #[test]
    fn apply() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::bytes_mut_max_length::tests::apply

        let s = "汤ДАЙЁ_35YuLx76xar";

        let mut bytes: BytesMut = BytesMut::new();
        bytes.write_str(&s).unwrap();

        let len: usize = bytes.len();

        assert!(BytesMutMaxLength::apply(&bytes, len + 2));
        assert!(!BytesMutMaxLength::apply(&bytes, len - 2));
    }
}
