pub mod auth;
pub mod errors;
pub mod home;
pub mod locale;
pub mod profile;
pub mod users;

use crate::{Alert, AlertVariant,  TranslatorService, ALERTS_KEY};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponseBuilder};
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
