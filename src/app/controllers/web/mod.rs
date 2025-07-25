pub mod auth;
pub mod errors;
pub mod files;
pub mod home;
pub mod locale;
pub mod profile;
pub mod roles;
pub mod user_files;
pub mod users;

use crate::{
    Alert, AlertVariant, AppService, FilePolicy, Locale, RolePolicy, RoleService, Session,
    TranslatorService, User, UserPolicy, WebAuthService, ALERTS_KEY,
};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponseBuilder};
use serde_json::{json, Value};

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
        let cookie: Vec<String> = alerts
            .into_iter()
            .map(|a| a.to_string().replace(",", "=-="))
            .collect();

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
    if s.len() > 4000 {
        return alerts;
    }
    for item in s.split(",") {
        let str = item.trim().replace("=-=", ",");
        let result = AlertVariant::from_string(&str);
        if let Ok(variant) = result {
            alerts.push(Alert::from_variant(translator_service, lang, &variant));
        }
    }
    alerts
}

pub struct ContextData<'a> {
    user: &'a User,
    translator_service: &'a TranslatorService,
    app_service: &'a AppService,
    role_service: &'a RoleService,
    dark_mode: Option<String>,
    lang: String,
    locale: &'a Locale,
    locales: &'a Vec<Locale>,
    csrf: String,
    alerts: Vec<Alert>,
    title: String,
    path: String,
}

pub fn get_context_data<'a>(
    req: &'a HttpRequest,
    user: &'a User,
    session: &'a Session,
    translator_service: &'a TranslatorService,
    app_service: &'a AppService,
    web_auth_service: &'a WebAuthService,
    role_service: &'a RoleService,
) -> ContextData<'a> {
    let path = req.path().to_string();
    let dark_mode: Option<String> = app_service.dark_mode(&req);
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(user));
    let csrf: String = web_auth_service.new_csrf(&session);
    let alerts: Vec<Alert> = req.get_alerts(&translator_service, &lang);
    let title = translator_service.translate(&lang, "app.name");
    ContextData {
        user,
        translator_service,
        app_service,
        dark_mode,
        lang,
        locale,
        locales,
        csrf,
        alerts,
        title,
        role_service,
        path,
    }
}

pub fn get_template_context<'a>(data: &'a ContextData) -> Value {
    let lang = &data.lang;
    let translator_service = data.translator_service;
    let app_service = data.app_service;
    let user = data.user;
    let role_service = data.role_service;

    let mut sidebar_users_index: Option<String> = None;
    let mut sidebar_roles_index: Option<String> = None;
    let mut sidebar_files: Option<String> = None;
    let mut is_sidebar_users_dropdown = false;

    if let Ok(roles) = role_service.all() {
        let is_users_show = UserPolicy::can_show(user, &roles);
        if is_users_show {
            sidebar_users_index =
                Some(translator_service.translate(lang, "layout.sidebar.users.index"));
        }
        let is_roles_show = RolePolicy::can_show(user, &roles);
        if is_roles_show {
            sidebar_roles_index =
                Some(translator_service.translate(lang, "layout.sidebar.users.roles"));
        }
        is_sidebar_users_dropdown = is_users_show && is_roles_show;

        if FilePolicy::can_show(user, &roles) {
            sidebar_files = Some(translator_service.translate(lang, "layout.sidebar.files"));
        }
    }

    json!({
        "site_url": app_service.url().to_string(),
        "title": &data.title,
        "brand": translator_service.translate(lang, "layout.brand"),
        "sidebar": {
            "home": translator_service.translate(lang, "layout.sidebar.home"),
            "users": {
              "is_dropdown": is_sidebar_users_dropdown,
              "index": sidebar_users_index,
              "roles": sidebar_roles_index,
            },
            "files": sidebar_files,
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
        "csrf": &data.csrf,
        "path": &data.path
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
pub fn generate_1_offset_pagination_array(page: i64, total_pages: i64) -> [i64; 7] {
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

#[allow(dead_code)]
pub fn generate_2_offset_pagination_array(page: i64, total_pages: i64) -> [i64; 9] {
    let mut result: [i64; 9] = [-1; 9];

    let result_length = 9;

    if result_length >= total_pages {
        for j in 0..total_pages {
            result[j as usize] = j + 1;
        }

        return result;
    }

    let mut i: usize = 0;

    result[i] = 1;
    i += 1;

    let mut start = page - 2;
    let mut end = page + 2;
    let mut is_start_dot = true;
    let mut is_end_dot = true;

    if start <= 3 {
        start = 2;
        end = 7;
        is_start_dot = false;
    }

    if end >= total_pages - 2 {
        if is_start_dot {
            start = total_pages - 6;
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

#[macro_export]
macro_rules! prepare_value {
    ($t:expr) => {
        if let Some(value) = &$t {
            let value_ = value.trim();
            if value_.len() == 0 {
                $t = None;
            } else if value_.len() != value.len() {
                $t = Some(value_.to_owned());
            }
        }
    };
}

#[macro_export]
macro_rules! prepare_upload_text_value {
    ($t:expr) => {
        if let Some(Text(value)) = &$t {
            let value_ = value.trim();
            if value_.len() == 0 {
                $t = None;
            } else if value_.len() != value.len() {
                $t = Some(Text(value_.to_owned()));
            }
        }
    };
}

#[macro_export]
macro_rules! assign_value_bytes_to_string {
    ($source:expr, $result:expr) => {
        {
            if $source.is_empty() {
                $result = None;
            } else {
                let value = String::from_utf8($source.to_vec()).map_err(|_| actix_web::error::ErrorBadRequest(""))?;
                let value = value.trim().to_string();
                if value.is_empty() {
                    $result = None;
                } else {
                    $result = Some(value);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! prepare_paginate {
    ($page:expr, $per_page:expr, $default_per_page:expr, $max_per_page:expr) => {
        let page = std::cmp::max($page.unwrap_or(1), 1);
        let per_page = std::cmp::min($per_page.unwrap_or($default_per_page), $max_per_page);
        $page = Some(page);
        $per_page = Some(per_page);
    };
}

#[macro_export]
macro_rules! validation_query_max_length_string {
    ($errors:expr, $field:expr, $field_name:expr, $max_size:expr, $translator_service:expr, $lang:expr) => {
        if let Some(value) = &$field {
            let mut errors_: Vec<String> =
                crate::app::validator::rules::str_max_chars_count::StrMaxCharsCount::validate(
                    $translator_service,
                    $lang,
                    value,
                    $max_size,
                    $field_name,
                );
            if errors_.len() != 0 {
                $errors.append(&mut errors_);
                $field = None;
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use generate_1_offset_pagination_array as ga;
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
