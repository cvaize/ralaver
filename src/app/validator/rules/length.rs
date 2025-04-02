use crate::{Translator, TranslatorVariable};

pub struct MinLengthString;
pub struct MaxLengthString;
pub struct MinMaxLengthString;

#[allow(dead_code)]
impl MinLengthString {
    pub fn apply(value: &String, min: usize) -> bool {
        value.len() >= min
    }

    pub fn validate(
        translator: &Translator,
        value: &String,
        min: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        if Self::apply(value, min) {
            vec![]
        } else {
            vec![translator.variables(
                "validation.min.string",
                vec![
                    TranslatorVariable::String(
                        "attribute".to_string(),
                        attribute_name.to_string(),
                    ),
                    TranslatorVariable::Usize("min".to_string(), min),
                ],
            )]
        }
    }
}

#[allow(dead_code)]
impl MaxLengthString {
    pub fn apply(value: &String, max: usize) -> bool {
        value.len() <= max
    }

    pub fn validate(
        translator: &Translator,
        value: &String,
        max: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        if Self::apply(value, max) {
            vec![]
        } else {
            vec![translator.variables(
                "validation.max.string",
                vec![
                    TranslatorVariable::String(
                        "attribute".to_string(),
                        attribute_name.to_string(),
                    ),
                    TranslatorVariable::Usize("max".to_string(), max),
                ],
            )]
        }
    }
}

#[allow(dead_code)]
impl MinMaxLengthString {
    pub fn apply(value: &String, min: usize, max: usize) -> bool {
        MinLengthString::apply(value, min) && MaxLengthString::apply(value, max)
    }

    pub fn validate(
        translator: &Translator,
        value: &String,
        min: usize,
        max: usize,
        attribute_name: &str,
    ) -> Vec<String> {
        let mut errors = MinLengthString::validate(translator, value, min, attribute_name);
        let mut errors2 = MaxLengthString::validate(translator, value, max, attribute_name);

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
        assert_eq!(true, MinMaxLengthString::apply(&"test_test".to_string(), 5, 15));
        assert_eq!(false, MinMaxLengthString::apply(&"test_test".to_string(), 5, 8));
    }
}
