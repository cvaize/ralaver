use crate::TranslatorService;

pub struct Confirmed;

#[allow(dead_code)]
impl Confirmed {
    pub fn apply<T: PartialEq>(a: &T, b: &T) -> bool {
        a.eq(b)
    }

    pub fn validate<T: PartialEq>(
        translator_service: &TranslatorService,
        lang: &str,
        a: &T,
        b: &T,
        attribute_name: &str,
    ) -> Vec<String> {
        if Self::apply(a, b) {
            Vec::new()
        } else {
            Vec::from([translator_service.var_str(
                translator_service
                    .translate(&lang, "validation.confirmed")
                    .as_str(),
                "attribute",
                attribute_name,
            )])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply() {
        assert_eq!(
            true,
            Confirmed::apply(&"test".to_string(), &"test".to_string())
        );
        assert_eq!(
            false,
            Confirmed::apply(&"test".to_string(), &"test2".to_string())
        );
    }
}
