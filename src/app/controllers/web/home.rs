use crate::{AppService, User};
use actix_web::web::Data;
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use crate::{AlertService, TemplateService};
use actix_session::Session;
use serde_json::json;

pub async fn index(
    req: HttpRequest,
    tmpl: Data<TemplateService>,
    alert_service: Data<AlertService>,
    session: Session,
    user: User,
    app_service: Data<AppService>,
) -> Result<HttpResponse, Error> {
    let alerts = alert_service
        .get_ref()
        .get_and_remove_from_session(&session)
        .unwrap_or(Vec::new());

    let dark_mode = app_service.get_ref().get_dark_mode(&req);

    let ctx = json!({
        "user" : user,
        "alerts": alerts,
        "dark_mode": dark_mode
    });
    let s = tmpl
        .get_ref()
        .render_throw_http("pages/home/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
