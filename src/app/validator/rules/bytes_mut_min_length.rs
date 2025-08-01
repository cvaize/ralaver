#![allow(dead_code)]
use crate::TranslatorService;
use std::collections::HashMap;
use bytes::BytesMut;

pub struct BytesMutMinLength;

impl BytesMutMinLength {
    pub fn apply(value: &BytesMut, min: usize) -> bool {
        value.len() >= min
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &BytesMut,
        min: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if !Self::apply(value, min) {
            let m = min.to_string();
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            vars.insert("min", m.as_str());
            v.push(translator_service.variables(&lang, "validation.min.file", &vars));
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
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::bytes_mut_min_length::tests::apply
        let s = "汤ДАЙЁ_35YuLx76xar";

        let mut bytes: BytesMut = BytesMut::new();
        bytes.write_str(&s).unwrap();

        let len: usize = bytes.len();

        assert!(BytesMutMinLength::apply(&bytes, len - 2));
        assert!(!BytesMutMinLength::apply(&bytes, len + 2));
    }
}
