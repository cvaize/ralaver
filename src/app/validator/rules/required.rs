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
    ) -> Vec<String> {
        if Self::apply(value) {
            Vec::new()
        } else {
            Vec::from([translator_service.translate(&lang, "validation.required")])
        }
    }

    pub fn validated<T, O: FnOnce(&T) -> Vec<String>>(
        translator_service: &TranslatorService,
        lang: &str,
        value: &Option<T>,
        cb: O,
    ) -> Vec<String> {
        if Self::apply(value) {
            cb(value.as_ref().unwrap())
        } else {
            Vec::from([translator_service.translate(&lang, "validation.required")])
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
