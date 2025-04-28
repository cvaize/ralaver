use std::collections::HashMap;
use crate::TranslatorService;

pub struct Email;

#[allow(dead_code)]
impl Email {
    pub fn apply(value: &String) -> bool {
        value.len() >= 3 && value.len() <= 255 && value.contains("@")
    }

    pub fn validate(
        translator_service: &TranslatorService,
        lang: &str,
        value: &String,
        attribute_name: &str,
    ) -> Vec<String> {
        if Self::apply(value) {
            Vec::new()
        } else {
            let mut vars = HashMap::new();
            vars.insert("attribute", attribute_name);
            Vec::from([translator_service.variables(&lang, "validation.email", &vars)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        assert_eq!(true, Email::apply(&"test@test".to_string()));
        assert_eq!(false, Email::apply(&"test".to_string()));
    }
}
