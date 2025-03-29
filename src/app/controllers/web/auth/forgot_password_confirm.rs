use actix_web::{error, web, Error, HttpResponse, Result};
use serde_json::Value::Null;
use crate::TemplateService;

pub async fn show(
    tmpl: web::Data<TemplateService>,
) -> Result<HttpResponse, Error> {
    let s = tmpl.get_ref().render("pages/auth/forgot-password-confirm.hbs", &Null)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
