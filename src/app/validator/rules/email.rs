use crate::TranslatorService;

pub struct Email;

#[allow(dead_code)]
impl Email {
    pub fn apply(value: &String) -> bool {
        value.len() >= 3 && value.len() <= 254 && value.contains("@")
    }

    pub fn validate(
        service: &TranslatorService,
        lang: &str,
        value: &Option<String>,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(value) = &value {
            if !Self::apply(value) {
                errors.push(
                    service
                        .translate(&lang, "validation.email")
                        .replace(":attribute", &attribute_name),
                );
            }
        } else {
            errors.push(service.translate(&lang, "validation.required"));
        }
        errors
    }
}
