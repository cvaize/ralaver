pub mod auth;
pub mod errors;
pub mod home;
pub mod locale;
pub mod profile;
pub mod users;

use crate::{model_redis_impl, Alert, AlertVariant, Translator, ALERTS_KEY};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponseBuilder};
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub struct FormData<Fields> {
    pub form: Option<DefaultForm<Fields>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultForm<Fields> {
    pub fields: Option<Fields>,
    pub errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultFields {
    pub email: Option<Field>,
    pub password: Option<Field>,
}

model_redis_impl!(DefaultFields);

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    pub value: Option<String>,
    pub errors: Option<Vec<String>>,
}

model_redis_impl!(Field);

impl<Fields> FormData<Fields> {
    pub fn empty() -> Self {
        Self { form: None }
    }
}

impl<Fields> DefaultForm<Fields> {
    pub fn empty() -> Self {
        Self {
            fields: None,
            errors: None,
        }
    }
}

// impl DefaultFields {
//     pub fn empty() -> Self {
//         Self {
//             email: None,
//             password: None,
//         }
//     }
// }

impl Field {
    pub fn empty() -> Self {
        Self {
            value: None,
            errors: None,
        }
    }
}

pub trait WebHttpRequest {
    fn get_alerts(&self, translator: &Translator) -> Vec<Alert>;
}

impl WebHttpRequest for HttpRequest {
    fn get_alerts(&self, translator: &Translator) -> Vec<Alert> {
        match self.cookie(ALERTS_KEY) {
            Some(cookie) => string_to_alerts(cookie.value(), translator),
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

fn string_to_alerts(s: &str, translator: &Translator) -> Vec<Alert> {
    let mut alerts = Vec::new();
    for item in s.split(",") {
        let result = AlertVariant::from_str(item.trim());
        if let Ok(variant) = result {
            alerts.push(Alert::from_variant(translator, &variant));
        }
    }
    alerts
}
