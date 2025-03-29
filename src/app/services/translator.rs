use crate::helpers::collect_files_from_dir;
use crate::Config;
use actix_web::web::Data;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use actix_session::Session;

#[derive(Debug, Clone)]
pub struct TranslatorService {
    config: Data<Config>,
    translates: HashMap<String, String>,
}

impl TranslatorService {
    pub fn new(config: Data<Config>, translates: HashMap<String, String>) -> Self {
        Self { config, translates }
    }

    pub fn new_from_files(config: Data<Config>) -> Result<Self, io::Error> {
        let mut translates: HashMap<String, String> = HashMap::from([]);

        let mut dir = env::current_dir()?;
        dir.push(Path::new(&config.get_ref().translator.translates_folder));
        let str_dir = dir.to_owned();
        let str_dir = str_dir.to_str().unwrap();

        let collect_paths: Vec<PathBuf> = collect_files_from_dir(dir.as_path())?;
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

            let content = fs::read_to_string(str_path)?;

            let flat_json: String = flatten_json::flatten_from_str(&content)?;
            let flatten_keys: Value = serde_json::from_str(&flat_json)?;

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

    // Функция возвращает перевод по переданному языку. Если перевод не найден по переданному языку,
    // то функция возвращает перевод по языку по умолчанию (app.locale). А если нет перевода по умолчанию, то берётся fallback язык (app.fallback_locale).
    // Если нет переводов с fallback языком, то возвращается переданный ключ.
    pub fn translate(&self, lang: &str, key: &str) -> String {
        if let Some(translate) = self.get(&format!("{}.{}", lang, key)) {
            return translate.to_string();
        }

        if lang != self.config.app.locale {
            if let Some(translate) = self.get(&format!("{}.{}", self.config.app.locale, key)) {
                return translate.to_string();
            }
        }

        if lang != self.config.app.fallback_locale
            && self.config.app.locale != self.config.app.fallback_locale
        {
            if let Some(translate) =
                self.get(&format!("{}.{}", self.config.app.fallback_locale, key))
            {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate() {
        let config = Data::new(Config::new_from_env());
        let t: TranslatorService = TranslatorService::new(
            config,
            HashMap::from([("en.test_key".to_string(), "test_value".to_string())]),
        );

        assert_eq!("test_value".to_string(), t.translate("en", "test_key"));
        assert_eq!("test_key".to_string(), t.translate("ru", "test_key"));
        assert_eq!("test_key123".to_string(), t.translate("en", "test_key123"));
    }

    #[test]
    fn new_from_files() {
        let config = Data::new(Config::new_from_env());
        let t: TranslatorService = TranslatorService::new_from_files(config).unwrap();
        let translates: &HashMap<String, String> = t.get_translates_ref();
        assert_ne!(0, translates.iter().len());
    }
}
