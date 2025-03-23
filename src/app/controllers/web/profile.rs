use crate::app::models::user::User;
use crate::app::services::session::{SessionFlashData, SessionFlashService};
use actix_web::{error, web, Error, HttpRequest, HttpResponse, Result};
use handlebars::Handlebars;
use serde_json::Value::Null;
use serde_json::{json, Value};

pub async fn index(
    req: HttpRequest,
    tmpl: web::Data<Handlebars<'_>>,
    user: User,
    flash_service: SessionFlashService,
) -> Result<HttpResponse, Error> {
    let flash_data: SessionFlashData = flash_service
        .read_and_forget(None)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    let user: Value = serde_json::to_value(&user).unwrap_or(Null);

    let ctx = json!({
            "user" : user,
            "alerts": flash_data.alerts,
            "dark_mode": req.cookie("dark_mode").map(|c| c.value().to_owned())
        });
    let s = tmpl.render("pages/profile/index.hbs", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}