use actix_web::{error, web, Error, HttpResponse, Result};
use serde_json::Value::Null;
use tinytemplate::TinyTemplate;

pub async fn login(
    tmpl: web::Data<TinyTemplate<'_>>
) -> Result<HttpResponse, Error> {
    let s = tmpl.render("pages.auth.login", &Null)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn register(
    tmpl: web::Data<TinyTemplate<'_>>
) -> Result<HttpResponse, Error> {
    let s = tmpl.render("pages.auth.register", &Null)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn forgot_password(
    tmpl: web::Data<TinyTemplate<'_>>
) -> Result<HttpResponse, Error> {
    let s = tmpl.render("pages.auth.forgot-password", &Null)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn forgot_password_confirm(
    tmpl: web::Data<TinyTemplate<'_>>
) -> Result<HttpResponse, Error> {
    let s = tmpl.render("pages.auth.forgot-password-confirm", &Null)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
