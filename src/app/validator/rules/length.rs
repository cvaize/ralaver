use crate::TranslatorService;

pub struct MinLengthString;
pub struct MaxLengthString;
pub struct MinMaxLengthString;

#[allow(dead_code)]
impl MinLengthString {
    pub fn apply(value: &String, min: usize) -> bool {
        value.len() >= min
    }

    pub fn validate(
        service: &TranslatorService,
        lang: &str,
        value: &Option<String>,
        min: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(value) = &value {
            if !Self::apply(value, min) {
                errors.push(
                    service
                        .translate(&lang, "validation.min.string")
                        .replace(":attribute", attribute_name)
                        .replace(":min", &min.to_string()),
                );
            }
        } else {
            errors.push(service.translate(&lang, "validation.required"));
        }
        errors
    }
}

#[allow(dead_code)]
impl MaxLengthString {
    pub fn apply(value: &String, max: usize) -> bool {
        value.len() <= max
    }

    pub fn validate(
        service: &TranslatorService,
        lang: &str,
        value: &Option<String>,
        max: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(value) = &value {
            if !Self::apply(value, max) {
                errors.push(
                    service
                        .translate(&lang, "validation.max.string")
                        .replace(":attribute", attribute_name)
                        .replace(":max", &max.to_string()),
                );
            }
        } else {
            errors.push(service.translate(&lang, "validation.required"));
        }
        errors
    }
}

#[allow(dead_code)]
impl MinMaxLengthString {
    pub fn apply(value: &String, min: usize, max: usize) -> bool {
        MinLengthString::apply(value, min) && MaxLengthString::apply(value, max)
    }

    pub fn validate(
        service: &TranslatorService,
        lang: &str,
        value: &Option<String>,
        min: usize,
        max: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(value) = &value {
            if !MinLengthString::apply(value, min) {
                errors.push(
                    service
                        .translate(&lang, "validation.min.string")
                        .replace(":attribute", attribute_name)
                        .replace(":min", &min.to_string()),
                );
            }
            if !MaxLengthString::apply(value, max) {
                errors.push(
                    service
                        .translate(&lang, "validation.max.string")
                        .replace(":attribute", attribute_name)
                        .replace(":max", &max.to_string()),
                );
            }
        } else {
            errors.push(service.translate(&lang, "validation.required"));
        }
        errors
    }
}
