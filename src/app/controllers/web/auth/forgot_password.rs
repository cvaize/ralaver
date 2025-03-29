use actix_web::{web, Error, HttpResponse, Result};
use serde_json::Value::Null;
use crate::TemplateService;

pub async fn show(
    tmpl: web::Data<TemplateService>
) -> Result<HttpResponse, Error> {
    let s = tmpl.get_ref().render_throw_http("pages/auth/forgot-password.hbs", &Null)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
