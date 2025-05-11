use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{Config, Locale, User};
use actix_web::web::Data;
use actix_web::HttpRequest;
use http::header::ACCEPT_LANGUAGE;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LocaleService {
    config: Data<Config>,
    locales: HashMap<String, Locale>,
    locales_codes: Vec<String>,
    locales_vec: Vec<Locale>,
}

impl LocaleService {
    pub fn new(config: Data<Config>) -> Self {
        let locales_vec = vec![
            Locale {
                code: "en".to_string(),
                short_name: "en".to_string(),
                full_name: "English".to_string(),
            },
            Locale {
                code: "ru".to_string(),
                short_name: "ru".to_string(),
                full_name: "Русский".to_string(),
            },
        ];
        let mut locales_codes: Vec<String> = Vec::new();
        let mut locales: HashMap<String, Locale> = HashMap::new();
        for locale in locales_vec.iter() {
            locales_codes.push(locale.code.to_string());
            locales.insert(locale.code.to_string(), locale.clone());
        }
        Self {
            config,
            locales,
            locales_codes,
            locales_vec,
        }
    }

    pub fn get_locale_ref(&self, code: &str) -> Option<&Locale> {
        self.locales.get(code)
    }

    pub fn get_locale_or_default_ref(&self, code: &str) -> &Locale {
        self.locales
            .get(code)
            .unwrap_or(self.locales.get(&self.config.get_ref().app.locale).unwrap())
    }

    pub fn get_default_ref(&self) -> &Locale {
        self.locales.get(&self.config.get_ref().app.locale).unwrap()
    }

    pub fn get_locales_ref(&self) -> &Vec<Locale> {
        &self.locales_vec
    }

    fn exists_locale_code_or_default(&self, key: String) -> String {
        if self.locales.contains_key(&key) {
            key
        } else {
            self.config.get_ref().app.locale.to_string()
        }
    }

    pub fn get_locales_codes_ref(&self) -> Vec<&str> {
        let mut codes = Vec::new();

        for locales_code in self.locales_codes.iter() {
            codes.push(locales_code.as_str());
        }

        codes
    }
    pub fn get_locale_code(&self, req: Option<&HttpRequest>, user: Option<&User>) -> String {
        if let Some(req) = req {
            if let Some(locale) = req.cookie(&self.config.get_ref().app.locale_cookie_key) {
                let locale = locale.value().to_string();
                if MinMaxLengthString::apply(&locale, 1, 6) {
                    return self.exists_locale_code_or_default(locale);
                }
            }
        }
        if let Some(user) = user {
            if let Some(locale) = &user.locale {
                if MinMaxLengthString::apply(&locale, 1, 6) {
                    return self.exists_locale_code_or_default(locale.to_string());
                }
            }
        }
        if let Some(req) = req {
            if let Some(header) = req.headers().get(ACCEPT_LANGUAGE) {
                let languages = accept_language::intersection(
                    header.to_str().unwrap_or(&self.config.get_ref().app.locale),
                    &self.get_locales_codes_ref(),
                );

                if let Some(locale) = languages.first() {
                    return self.exists_locale_code_or_default(locale.to_string());
                }
            }
        }

        self.config.get_ref().app.locale.to_string()
    }
}
