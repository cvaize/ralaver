use crate::app::validator::rules::str_min_max_chars_count::StrMinMaxCharsCount;
use crate::Config;
use actix_web::cookie::Cookie;
use actix_web::web::{Data, Form};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result, http::{header::{ORIGIN, REFERER, LOCATION, HeaderValue}}};
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct LocaleData {
    pub locale: Option<String>,
}

pub async fn switch(
    req: HttpRequest,
    data: Form<LocaleData>,
    config: Data<Config>,
) -> Result<HttpResponse, Error> {
    let config = config.get_ref();
    if data.locale.is_none() {
        return Err(error::ErrorBadRequest(""));
    }

    let locale = match &data.locale {
        Some(l) => l.to_string(),
        _ => config.app.locale.to_string(),
    };

    if !StrMinMaxCharsCount::apply(&locale, 1, 6) {
        return Err(error::ErrorBadRequest(""));
    }

    let c = Cookie::build(&config.app.locale_cookie_key, locale)
        .path("/")
        .http_only(true)
        .finish();

    let headers = req.headers();
    let default = HeaderValue::from_static("/");
    let location = headers.get(REFERER).unwrap_or(headers.get(ORIGIN).unwrap_or(&default));
    let location = location.to_str().unwrap_or("/");

    Ok(HttpResponse::SeeOther()
        .cookie(c)
        .insert_header((LOCATION, HeaderValue::from_str(location).unwrap_or(default)))
        .finish())
}
