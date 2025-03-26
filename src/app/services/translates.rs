use std::collections::HashMap;


#[derive(Debug)]
pub struct Translates {
    map: HashMap<String, String>,
}

impl Translates {
    pub fn new(map: HashMap<String, String>) -> Self {
        Self { map }
    }

    pub fn value(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    pub fn value_or_key(&self, key: &str) -> String {
        if let Some(translate) = self.map.get(key) {
            translate.to_string()
        } else {
            key.to_string()
        }
    }

    pub fn map(&self) -> &HashMap<String, String> {
        &self.map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn translate() {
        let service = Translates::new(HashMap::from([(
            "en.test_key".to_string(),
            "test_value".to_string(),
        )]));

        assert_eq!("test_value".to_string(), service.value_or_key("en.test_key"));
        assert_eq!("ru.test_key".to_string(), service.value_or_key("ru.test_key"));
        assert_eq!(
            "en.test_key123".to_string(),
            service.value_or_key("en.test_key123")
        );
    }

    #[test]
    fn map() {
        let service = Translates::new(HashMap::from([(
            "en.test_key".to_string(),
            "test_value".to_string(),
        )]));
        assert_eq!(1, service.map().iter().len());
    }
}
