#![allow(dead_code)]
use crate::helpers::join_array;
use crate::TranslatorService;
use bytes::Bytes;
use mime::Mime;
use std::collections::HashMap;

pub struct Mimes;

impl Mimes {
    pub fn apply(value: &Option<Mime>, mimes: &[Mime]) -> bool {
        if let Some(value) = value {
            return mimes.contains(value);
        }
        false
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &Option<Mime>,
        mimes: &[Mime],
        attribute_name: &str,
    ) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if !Self::apply(value, mimes) {
            let values = join_array(mimes, ", ");
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            vars.insert("values", values.as_str());
            v.push(translator_service.variables(&lang, "validation.mimes", &vars));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mime::{IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG};

    #[test]
    fn apply() {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::validator::rules::mimes::tests::apply

        assert!(Mimes::apply(&Some(IMAGE_JPEG), &[IMAGE_JPEG, IMAGE_PNG]));
        assert!(Mimes::apply(&Some(IMAGE_PNG), &[IMAGE_JPEG, IMAGE_PNG]));
        assert!(!Mimes::apply(&Some(IMAGE_GIF), &[IMAGE_JPEG, IMAGE_PNG]));
        assert!(!Mimes::apply(&None, &[IMAGE_JPEG, IMAGE_PNG]));
    }
}
