use std::collections::HashMap;
use crate::TranslatorService;

pub struct Required;

#[allow(dead_code)]
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
        if Self::apply(value) {
            Vec::new()
        } else {
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            Vec::from([translator_service.variables(&lang, "validation.required", &vars)])
        }
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
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            Vec::from([translator_service.variables(&lang, "validation.required", &vars)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        let value: Option<String> = None;
        assert_eq!(true, Required::apply(&Some("test".to_string())));
        assert_eq!(false, Required::apply(&value));
    }
}
