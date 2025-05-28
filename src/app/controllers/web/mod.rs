pub mod auth;
pub mod errors;
pub mod home;
pub mod locale;
pub mod profile;
pub mod users;

use crate::{
    Alert, AlertVariant, AppService, Locale, Session, TranslatorService, User,
    WebAuthService, ALERTS_KEY,
};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponseBuilder};
use rand::Rng;
use serde_json::{json, Value};
use std::str::FromStr;

pub trait WebHttpRequest {
    fn get_alerts(&self, translator_service: &TranslatorService, lang: &str) -> Vec<Alert>;
}

impl WebHttpRequest for HttpRequest {
    fn get_alerts(&self, translator_service: &TranslatorService, lang: &str) -> Vec<Alert> {
        match self.cookie(ALERTS_KEY) {
            Some(cookie) => string_to_alerts(cookie.value(), translator_service, lang),
            _ => Vec::new(),
        }
    }
}

pub trait WebHttpResponse {
    fn clear_alerts(&mut self) -> &mut Self;
    fn set_alerts(&mut self, alerts: Vec<AlertVariant>) -> &mut Self;
}

impl WebHttpResponse for HttpResponseBuilder {
    fn clear_alerts(&mut self) -> &mut Self {
        self.cookie(
            Cookie::build(ALERTS_KEY, "")
                .path("/")
                .http_only(true)
                .secure(false)
                .max_age(Duration::seconds(0))
                .finish(),
        )
    }
    fn set_alerts(&mut self, alerts: Vec<AlertVariant>) -> &mut Self {
        let cookie: Vec<String> = alerts.into_iter().map(|a| a.to_string()).collect();

        self.cookie(
            Cookie::build(ALERTS_KEY, cookie.join(","))
                .path("/")
                .http_only(true)
                .secure(false)
                .max_age(Duration::seconds(600))
                .finish(),
        )
    }
}

fn string_to_alerts(s: &str, translator_service: &TranslatorService, lang: &str) -> Vec<Alert> {
    let mut alerts = Vec::new();
    for item in s.split(",") {
        let result = AlertVariant::from_str(item.trim());
        if let Ok(variant) = result {
            alerts.push(Alert::from_variant(translator_service, lang, &variant));
        }
    }
    alerts
}

pub struct ContextData<'a> {
    user: &'a User,
    translator_service: &'a TranslatorService,
    dark_mode: Option<String>,
    lang: String,
    locale: &'a Locale,
    locales: &'a Vec<Locale>,
    csrf: String,
    alerts: Vec<Alert>,
    title: String,
}

pub fn get_context_data<'a>(
    req: &'a HttpRequest,
    user: &'a User,
    session: &'a Session,
    translator_service: &'a TranslatorService,
    app_service: &'a AppService,
    web_auth_service: &'a WebAuthService,
) -> ContextData<'a> {
    let dark_mode: Option<String> = app_service.dark_mode(&req);
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(user));
    let csrf: String = web_auth_service.new_csrf(&session);
    let alerts: Vec<Alert> = req.get_alerts(&translator_service, &lang);
    let title = translator_service.translate(&lang, "app.name");
    ContextData {
        user,
        translator_service,
        dark_mode,
        lang,
        locale,
        locales,
        csrf,
        alerts,
        title,
    }
}

pub fn get_template_context<'a>(data: &'a ContextData) -> Value {
    let lang = &data.lang;
    let translator_service = data.translator_service;
    json!({
      "title": &data.title,
      "brand": translator_service.translate(lang, "layout.brand"),
      "sidebar": {
        "home": translator_service.translate(lang, "layout.sidebar.home"),
        "users": {
          "index": translator_service.translate(lang, "layout.sidebar.users.index"),
          "roles": translator_service.translate(lang, "layout.sidebar.users.roles"),
          "permissions": translator_service.translate(lang, "layout.sidebar.users.permissions"),
        },
        "profile": translator_service.translate(lang, "layout.sidebar.profile"),
        "logout": translator_service.translate(lang, "layout.sidebar.logout"),
      },
      "dark_mode": {
        "value": &data.dark_mode,
        "dark": translator_service.translate(lang, "layout.dark_mode.dark"),
        "light": translator_service.translate(lang, "layout.dark_mode.light"),
        "auto": translator_service.translate(lang, "layout.dark_mode.auto"),
      },
      "locale": &data.locale,
      "locales": &data.locales,
      "user" : &data.user,
      "alerts": &data.alerts,
      "csrf": &data.csrf
    })
}

pub struct PublicContextData<'a> {
    translator_service: &'a TranslatorService,
    dark_mode: Option<String>,
    lang: String,
    locale: &'a Locale,
    locales: &'a Vec<Locale>,
    alerts: Vec<Alert>,
    title: String,
}

pub fn get_public_context_data<'a>(
    req: &'a HttpRequest,
    translator_service: &'a TranslatorService,
    app_service: &'a AppService,
) -> PublicContextData<'a> {
    let dark_mode: Option<String> = app_service.dark_mode(&req);
    let (lang, locale, locales) = app_service.locale(Some(&req), None);
    let alerts: Vec<Alert> = req.get_alerts(&translator_service, &lang);
    let title = translator_service.translate(&lang, "app.name");
    PublicContextData {
        translator_service,
        dark_mode,
        lang,
        locale,
        locales,
        alerts,
        title,
    }
}

pub fn get_public_template_context<'a>(data: &'a PublicContextData) -> Value {
    let lang = &data.lang;
    let translator_service = data.translator_service;
    json!({
      "title": &data.title,
      "dark_mode": {
        "value": &data.dark_mode,
        "dark": translator_service.translate(lang, "layout.dark_mode.dark"),
        "light": translator_service.translate(lang, "layout.dark_mode.light"),
        "auto": translator_service.translate(lang, "layout.dark_mode.auto"),
      },
      "locale": &data.locale,
      "locales": &data.locales,
      "alerts": &data.alerts,
    })
}

#[allow(dead_code)]
pub fn generate_pagination_vec(page: i64, total_pages: i64, offset: i64) -> Vec<i64> {
    let mut result: Vec<i64> = Vec::new();

    let result_length = 5 + offset * 2;

    if result_length >= total_pages {
        for j in 1..=total_pages {
            result.push(j);
        }

        return result;
    }

    result.push(1);

    let mut start = page - offset;
    let mut end = page + offset;
    let mut is_start_dot = true;
    let mut is_end_dot = true;

    if start <= 3 {
        start = 2;
        end = 3 + offset * 2;
        is_start_dot = false;
    }

    if end >= total_pages - 2 {
        if is_start_dot {
            start = total_pages - (2 + offset * 2);
        }

        end = total_pages - 1;
        is_end_dot = false;
    }

    if start <= 3 {
        start = 2;
        is_start_dot = false;
    }

    if is_start_dot {
        result.push(0);
    }

    for j in start..=end {
        result.push(j);
    }

    if is_end_dot {
        result.push(0);
    }

    result.push(total_pages);

    result
}

#[allow(dead_code)]
pub fn generate_pagination_array(page: i64, total_pages: i64) -> [i64; 7] {
    let mut result: [i64; 7] = [-1; 7];

    let result_length = 7;

    if result_length >= total_pages {
        for j in 0..total_pages {
            result[j as usize] = j + 1;
        }

        return result;
    }

    let mut i: usize = 0;

    result[i] = 1;
    i += 1;

    let mut start = page - 1;
    let mut end = page + 1;
    let mut is_start_dot = true;
    let mut is_end_dot = true;

    if start <= 3 {
        start = 2;
        end = 5;
        is_start_dot = false;
    }

    if end >= total_pages - 2 {
        if is_start_dot {
            start = total_pages - 4;
        }

        end = total_pages - 1;
        is_end_dot = false;
    }

    if start <= 3 {
        start = 2;
        is_start_dot = false;
    }

    if is_start_dot {
        result[i] = 0;
        i += 1;
    }

    for j in start..=end {
        result[i] = j;
        i += 1;
    }

    if is_end_dot {
        result[i] = 0;
        i += 1;
    }

    result[i] = total_pages;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use generate_pagination_array as ga;
    use generate_pagination_vec as gv;
    use test::Bencher;

    #[test]
    fn test_generate_pagination_vec() {
        assert_eq!(gv(-11, 5, 1), vec![1, 2, 3, 4, 5]);
        assert_eq!(gv(1, 5, 1), vec![1, 2, 3, 4, 5]);
        assert_eq!(gv(2, 5, 1), vec![1, 2, 3, 4, 5]);
        assert_eq!(gv(3, 5, 1), vec![1, 2, 3, 4, 5]);
        assert_eq!(gv(4, 5, 1), vec![1, 2, 3, 4, 5]);
        assert_eq!(gv(5, 5, 1), vec![1, 2, 3, 4, 5]);
        assert_eq!(gv(11, 5, 1), vec![1, 2, 3, 4, 5]);

        assert_eq!(gv(-111, 10, 1), vec![1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(gv(1, 10, 1), vec![1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(gv(2, 10, 1), vec![1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(gv(3, 10, 1), vec![1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(gv(4, 10, 1), vec![1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(gv(5, 10, 1), vec![1, 0, 4, 5, 6, 0, 10]);
        assert_eq!(gv(6, 10, 1), vec![1, 0, 5, 6, 7, 0, 10]);
        assert_eq!(gv(7, 10, 1), vec![1, 0, 6, 7, 8, 9, 10]);
        assert_eq!(gv(8, 10, 1), vec![1, 0, 6, 7, 8, 9, 10]);
        assert_eq!(gv(9, 10, 1), vec![1, 0, 6, 7, 8, 9, 10]);
        assert_eq!(gv(10, 10, 1), vec![1, 0, 6, 7, 8, 9, 10]);
        assert_eq!(gv(111, 10, 1), vec![1, 0, 6, 7, 8, 9, 10]);

        assert_eq!(gv(-222, 20, 2), vec![1, 2, 3, 4, 5, 6, 7, 0, 20]);
        assert_eq!(gv(1, 20, 2), vec![1, 2, 3, 4, 5, 6, 7, 0, 20]);
        assert_eq!(gv(2, 20, 2), vec![1, 2, 3, 4, 5, 6, 7, 0, 20]);
        assert_eq!(gv(3, 20, 2), vec![1, 2, 3, 4, 5, 6, 7, 0, 20]);
        assert_eq!(gv(4, 20, 2), vec![1, 2, 3, 4, 5, 6, 7, 0, 20]);
        assert_eq!(gv(5, 20, 2), vec![1, 2, 3, 4, 5, 6, 7, 0, 20]);
        assert_eq!(gv(6, 20, 2), vec![1, 0, 4, 5, 6, 7, 8, 0, 20]);
        assert_eq!(gv(7, 20, 2), vec![1, 0, 5, 6, 7, 8, 9, 0, 20]);
        assert_eq!(gv(8, 20, 2), vec![1, 0, 6, 7, 8, 9, 10, 0, 20]);
        assert_eq!(gv(9, 20, 2), vec![1, 0, 7, 8, 9, 10, 11, 0, 20]);
        assert_eq!(gv(10, 20, 2), vec![1, 0, 8, 9, 10, 11, 12, 0, 20]);
        assert_eq!(gv(11, 20, 2), vec![1, 0, 9, 10, 11, 12, 13, 0, 20]);
        assert_eq!(gv(12, 20, 2), vec![1, 0, 10, 11, 12, 13, 14, 0, 20]);
        assert_eq!(gv(13, 20, 2), vec![1, 0, 11, 12, 13, 14, 15, 0, 20]);
        assert_eq!(gv(14, 20, 2), vec![1, 0, 12, 13, 14, 15, 16, 0, 20]);
        assert_eq!(gv(15, 20, 2), vec![1, 0, 13, 14, 15, 16, 17, 0, 20]);
        assert_eq!(gv(16, 20, 2), vec![1, 0, 14, 15, 16, 17, 18, 19, 20]);
        assert_eq!(gv(17, 20, 2), vec![1, 0, 14, 15, 16, 17, 18, 19, 20]);
        assert_eq!(gv(18, 20, 2), vec![1, 0, 14, 15, 16, 17, 18, 19, 20]);
        assert_eq!(gv(19, 20, 2), vec![1, 0, 14, 15, 16, 17, 18, 19, 20]);
        assert_eq!(gv(20, 20, 2), vec![1, 0, 14, 15, 16, 17, 18, 19, 20]);
        assert_eq!(gv(222, 20, 2), vec![1, 0, 14, 15, 16, 17, 18, 19, 20]);
    }

    #[test]
    fn test_generate_pagination_array() {
        assert_eq!(ga(-11, 5), [1, 2, 3, 4, 5, -1, -1]);
        assert_eq!(ga(1, 5), [1, 2, 3, 4, 5, -1, -1]);
        assert_eq!(ga(2, 5), [1, 2, 3, 4, 5, -1, -1]);
        assert_eq!(ga(3, 5), [1, 2, 3, 4, 5, -1, -1]);
        assert_eq!(ga(4, 5), [1, 2, 3, 4, 5, -1, -1]);
        assert_eq!(ga(5, 5), [1, 2, 3, 4, 5, -1, -1]);
        assert_eq!(ga(11, 5), [1, 2, 3, 4, 5, -1, -1]);

        assert_eq!(ga(-111, 10), [1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(ga(1, 10), [1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(ga(2, 10), [1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(ga(3, 10), [1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(ga(4, 10), [1, 2, 3, 4, 5, 0, 10]);
        assert_eq!(ga(5, 10), [1, 0, 4, 5, 6, 0, 10]);
        assert_eq!(ga(6, 10), [1, 0, 5, 6, 7, 0, 10]);
        assert_eq!(ga(7, 10), [1, 0, 6, 7, 8, 9, 10]);
        assert_eq!(ga(8, 10), [1, 0, 6, 7, 8, 9, 10]);
        assert_eq!(ga(9, 10), [1, 0, 6, 7, 8, 9, 10]);
        assert_eq!(ga(10, 10), [1, 0, 6, 7, 8, 9, 10]);
        assert_eq!(ga(111, 10), [1, 0, 6, 7, 8, 9, 10]);
    }

    #[bench]
    fn bench_generate_pagination_vec(b: &mut Bencher) {
        // 38.12 ns/iter (+/- 2.45)
        b.iter(|| gv(6, 10, 1));
    }

    #[bench]
    fn bench_generate_pagination_array(b: &mut Bencher) {
        // 0.88 ns/iter (+/- 0.01)
        b.iter(|| ga(6, 10))
    }
}
