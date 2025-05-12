pub mod auth;
pub mod errors;
pub mod home;
pub mod locale;
pub mod profile;
pub mod users;

use crate::{
    Alert, AlertVariant, AppService, Locale, Session, TranslatorService, User, WebAuthService,
    ALERTS_KEY,
};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponseBuilder};
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
