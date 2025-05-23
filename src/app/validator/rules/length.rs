use crate::TranslatorService;
use std::collections::HashMap;

pub struct MinLengthString;
pub struct MaxLengthString;
pub struct MinMaxLengthString;

#[allow(dead_code)]
impl MinLengthString {
    pub fn apply(value: &String, min: usize) -> bool {
        value.len() >= min
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &String,
        min: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        if Self::apply(value, min) {
            Vec::new()
        } else {
            let m = min.to_string();
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            vars.insert("min", m.as_str());
            Vec::from([translator_service.variables(&lang, "validation.min.string", &vars)])
        }
    }
}

#[allow(dead_code)]
impl MaxLengthString {
    pub fn apply(value: &str, max: usize) -> bool {
        value.len() <= max
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &str,
        max: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        if Self::apply(value, max) {
            Vec::new()
        } else {
            let m = max.to_string();
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            vars.insert("max", m.as_str());
            Vec::from([translator_service.variables(&lang, "validation.max.string", &vars)])
        }
    }
}

#[allow(dead_code)]
impl MinMaxLengthString {
    pub fn apply(value: &String, min: usize, max: usize) -> bool {
        MinLengthString::apply(value, min) && MaxLengthString::apply(value, max)
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &String,
        min: usize,
        max: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut errors =
            MinLengthString::validate(translator_service, lang, value, min, attribute_name);
        let mut errors2 =
            MaxLengthString::validate(translator_service, lang, value, max, attribute_name);

        errors.append(&mut errors2);
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        assert_eq!(true, MinLengthString::apply(&"test_test".to_string(), 5));
        assert_eq!(false, MinLengthString::apply(&"test".to_string(), 5));
        assert_eq!(false, MaxLengthString::apply(&"test_test".to_string(), 5));
        assert_eq!(true, MaxLengthString::apply(&"test".to_string(), 5));
        assert_eq!(
            true,
            MinMaxLengthString::apply(&"test_test".to_string(), 5, 15)
        );
        assert_eq!(
            false,
            MinMaxLengthString::apply(&"test_test".to_string(), 5, 8)
        );
    }
}
