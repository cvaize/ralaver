use crate::{Translator, TranslatorVariable};

pub struct Confirmed;

#[allow(dead_code)]
impl Confirmed {
    pub fn apply<T: PartialEq>(a: &T, b: &T) -> bool {
        a.eq(b)
    }

    pub fn validate<T: PartialEq>(
        translator: &Translator,
        a: &T,
        b: &T,
        attribute_name: &str,
    ) -> Vec<String> {
        if Self::apply(a, b) {
            vec![]
        } else {
            vec![translator.variables(
                "validation.confirmed",
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
