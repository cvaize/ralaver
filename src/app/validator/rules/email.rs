use crate::{TranslatorService, TranslatorVariable};

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
            vec![]
        } else {
            vec![translator_service.variables(
                &lang,
                "validation.email",
                vec![TranslatorVariable::String(
                    "attribute".to_string(),
                    attribute_name.to_string(),
                )],
            )]
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
