use actix_web::{error, web, Error, HttpResponse, Result};
use serde_json::Value::Null;
use tinytemplate::TinyTemplate;

pub async fn show(
    tmpl: web::Data<TinyTemplate<'_>>
) -> Result<HttpResponse, Error> {
    let s = tmpl.render("pages.auth.register", &Null)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn store(
    tmpl: web::Data<TinyTemplate<'_>>
) -> Result<HttpResponse, Error> {
    let s = tmpl.render("pages.auth.register", &Null)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}