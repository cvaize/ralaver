use crate::helpers::collect_files_from_dir;
use crate::{Config};
use actix_web::web::Data;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

#[derive(Debug, Clone)]
pub struct TranslatorService {
    config: Data<Config>,
    translates: HashMap<String, String>,
}

impl TranslatorService {
    pub fn new(
        config: Data<Config>,
        translates: HashMap<String, String>,
    ) -> Self {
        Self {
            config,
            translates,
        }
    }

    pub fn new_from_files(
        config: Data<Config>,
    ) -> Result<Self, io::Error> {
        let mut translates: HashMap<String, String> = HashMap::from([]);

        let mut dir = env::current_dir().map_err(|e| {
            log::error!("{}",format!("TranslatorService::new_from_files - {:}", &e).as_str());
            e
        })?;
        dir.push(Path::new(&config.get_ref().translator.translates_folder));
        let str_dir = dir.to_owned();
        let str_dir = str_dir.to_str().unwrap();

        let collect_paths: Vec<PathBuf> = collect_files_from_dir(dir.as_path()).map_err(|e| {
            log::error!("{}",format!("TranslatorService::new_from_files - {:}", &e).as_str());
            e
        })?;
        let paths: Vec<&PathBuf> = collect_paths
            .iter()
            .filter(|&p| p.extension().unwrap() == "json")
            .collect::<Vec<&PathBuf>>();

        for path in paths {
            let str_path = path.to_str().unwrap();
            let replace_str = format!("{}/", str_dir);
            let replace_str = replace_str.as_str();
            let name = str_path
                .replace(replace_str, "")
                .replace(".json", "")
                .replace("/", ".");

            let content = fs::read_to_string(str_path).map_err(|e| {
                log::error!("{}",format!("TranslatorService::new_from_files - {:}", &e).as_str());
                e
            })?;

            let flat_json: String = flatten_json::flatten_from_str(&content).map_err(|e| {
                log::error!("{}",format!("TranslatorService::new_from_files - {:}", &e).as_str());
                e
            })?;
            let flatten_keys: Value = serde_json::from_str(&flat_json).map_err(|e| {
                log::error!("{}",format!("TranslatorService::new_from_files - {:}", &e).as_str());
                e
            })?;

            for (key, value) in flatten_keys.as_object().unwrap().iter() {
                let mut full_key: String = name.clone();
                full_key.push_str(".");
                full_key.push_str(key);
                if let Some(value) = value.as_str() {
                    translates.insert(full_key.to_string(), value.to_string());
                }
            }
        }

        Ok(Self::new(config, translates))
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.translates.get(key)
    }

    pub fn get_or_key(&self, key: &str) -> String {
        if let Some(translate) = self.get(key) {
            translate.to_string()
        } else {
            key.to_string()
        }
    }

    pub fn apply_variables(&self, string: String, variables: Vec<TranslatorVariable>) -> String {
        let mut string: String = string;
        for variable in &variables {
            string = match variable {
                TranslatorVariable::Usize(key, value) => {
                    string.replace(&format!(":{}", key), value.to_string().as_str())
                }
                TranslatorVariable::I32(key, value) => {
                    string.replace(&format!(":{}", key), value.to_string().as_str())
                }
                TranslatorVariable::U64(key, value) => {
                    string.replace(&format!(":{}", key), value.to_string().as_str())
                }
                TranslatorVariable::String(key, value) => {
                    string.replace(&format!(":{}", key), value)
                }
            }
        }
        string
    }

    pub fn translate_with_variables(
        &self,
        lang: &str,
        key: &str,
        variables: Vec<TranslatorVariable>,
    ) -> String {
        self.apply_variables(self.translate(lang, key), variables)
    }

    // Функция возвращает перевод по переданному языку. Если перевод не найден по переданному языку,
    // то функция возвращает перевод по языку по умолчанию (app.locale). А если нет перевода по умолчанию, то берётся fallback язык (app.fallback_locale).
    // Если нет переводов с fallback языком, то возвращается переданный ключ.
    pub fn translate(&self, lang: &str, key: &str) -> String {
        if let Some(translate) = self.get(&format!("{}.{}", lang, key)) {
            return translate.to_string();
        }

        let app = &self.config.get_ref().app;
        if lang != app.locale {
            if let Some(translate) = self.get(&format!("{}.{}", app.locale, key)) {
                return translate.to_string();
            }
        }

        if lang != app.fallback_locale && app.locale != app.fallback_locale {
            if let Some(translate) = self.get(&format!("{}.{}", app.fallback_locale, key)) {
                return translate.to_string();
            }
        }

        key.to_string()
    }

    pub fn hard_translate(&self, lang: &str, key: &str) -> Option<&String> {
        self.get(&format!("{}.{}", lang, key))
    }

    pub fn soft_translate(&self, lang: &str, key: &str) -> String {
        if let Some(translate) = self.get(&format!("{}.{}", lang, key)) {
            translate.to_string()
        } else {
            key.to_string()
        }
    }

    pub fn get_translates_ref(&self) -> &HashMap<String, String> {
        &self.translates
    }
}

pub struct Translator<'a> {
    lang: String,
    translator_service: &'a TranslatorService,
}

impl<'a> Translator<'a> {
    pub fn new(lang: &str, translator_service: &'a TranslatorService) -> Self {
        Self {
            lang: lang.to_string(),
            translator_service,
        }
    }
    pub fn simple(&self, key: &str) -> String {
        self.translator_service.translate(self.lang.as_str(), key)
    }
    pub fn variables(&self, key: &str, variables: Vec<TranslatorVariable>) -> String {
        self.translator_service
            .translate_with_variables(self.lang.as_str(), key, variables)
    }
}

pub enum TranslatorVariable {
    String(String, String),
    I32(String, i32),
    U64(String, u64),
    Usize(String, usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate() {
        let config = Data::new(Config::new());
        let t: TranslatorService = TranslatorService::new(
            config,
            HashMap::from([
                ("en.test_key".to_string(), "test_value".to_string()),
                ("en.test_key2".to_string(), "test_value2".to_string()),
            ]),
        );

        assert_eq!("test_value".to_string(), t.translate("en", "test_key"));
        assert_eq!("test_value".to_string(), t.translate("fi", "test_key"));
        assert_eq!("test_key123".to_string(), t.translate("en", "test_key123"));
    }

    #[test]
    fn new_from_files() {
        let config = Data::new(Config::new());
        let t: TranslatorService = TranslatorService::new_from_files(config).unwrap();
        let translates: &HashMap<String, String> = t.get_translates_ref();
        assert_ne!(0, translates.iter().len());
    }
}
