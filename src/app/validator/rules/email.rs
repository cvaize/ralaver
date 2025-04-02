use crate::{Translator, TranslatorVariable};

pub struct Email;

#[allow(dead_code)]
impl Email {
    pub fn apply(value: &String) -> bool {
        value.len() >= 3 && value.len() <= 254 && value.contains("@")
    }

    pub fn validate(
        translator: &Translator,
        value: &Option<String>,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(value) = &value {
            if !Self::apply(value) {
                errors.push(translator.variables(
                    "validation.email",
                    vec![TranslatorVariable::String(
                        "attribute".to_string(),
                        attribute_name.to_string(),
                    )],
                ));
            }
        } else {
            errors.push(translator.simple("validation.required"));
        }
        errors
    }
}
