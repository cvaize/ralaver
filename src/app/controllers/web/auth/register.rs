use actix_web::web::Data;
use actix_web::{Error, HttpResponse, Result};
use serde_json::Value::Null;
use crate::TemplateService;

pub async fn show(
    tmpl: Data<TemplateService>,
) -> Result<HttpResponse, Error> {
    let s = tmpl.get_ref().render_throw_http("pages/auth/register.hbs", &Null)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn store(
    tmpl: Data<TemplateService>,
) -> Result<HttpResponse, Error> {
    let s = tmpl.get_ref().render_throw_http("pages/auth/register.hbs", &Null)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}