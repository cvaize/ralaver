use crate::Config;
use actix_web::cookie::Cookie;
use actix_web::web::{Data, Form};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use garde::Validate;
use http::header::{ORIGIN, REFERER};
use http::HeaderValue;
use serde_derive::Deserialize;

#[derive(Validate, Deserialize, Debug)]
pub struct LocaleData {
    #[garde(required, inner(length(min = 1, max = 6)))]
    pub locale: Option<String>,
}

pub async fn switch(
    req: HttpRequest,
    data: Form<LocaleData>,
    config: Data<Config>,
) -> Result<HttpResponse, Error> {
    data.validate()
        .map_err(|_| error::ErrorBadRequest("Validate error"))?;

    let locale = match &data.locale {
        Some(l) => l.to_string(),
        _ => config.get_ref().app.locale.to_string(),
    };
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
