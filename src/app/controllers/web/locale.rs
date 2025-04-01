use crate::Config;
use actix_web::cookie::Cookie;
use actix_web::web::{Data, Form};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::header::{ORIGIN, REFERER};
use http::HeaderValue;
use serde_derive::Deserialize;
use crate::app::validator::rules::length::MinMaxLengthString;

#[derive(Deserialize, Debug)]
pub struct LocaleData {
    pub locale: Option<String>,
}

pub async fn switch(
    req: HttpRequest,
    data: Form<LocaleData>,
    config: Data<Config>,
) -> Result<HttpResponse, Error> {
    if data.locale.is_none() {
        return Err(error::ErrorBadRequest("Validate error"));
    }

    let locale = match &data.locale {
        Some(l) => l.to_string(),
        _ => config.get_ref().app.locale.to_string(),
    };

    if !MinMaxLengthString::apply(&locale, 1, 6) {
        return Err(error::ErrorBadRequest("Validate error"));
    }

    let c = Cookie::build(&config.get_ref().app.locale_cookie_key, locale)
        .path("/")
        .http_only(true)
        .finish();

    let headers = req.headers();
    let default = HeaderValue::from_static("/");
    let location = headers.get(ORIGIN).unwrap_or(headers.get(REFERER).unwrap_or(&default));
    let location = location.to_str().unwrap_or("/");

    Ok(HttpResponse::SeeOther()
        .cookie(c)
        .insert_header((http::header::LOCATION, HeaderValue::from_str(location).unwrap_or(default)))
        .finish())
}
