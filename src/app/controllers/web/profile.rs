use crate::{AppService, TemplateService, User};
use actix_session::Session;
use actix_web::web::Data;
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use serde_json::json;

pub async fn index(
    req: HttpRequest,
    tmpl: Data<TemplateService>,
    app_service: Data<AppService>,
    session: Session,
    user: User,
) -> Result<HttpResponse, Error> {
    let alerts = app_service.get_ref().alerts(&session);

    let dark_mode = app_service.get_ref().dark_mode(&req);

    let (_, locale, locales) = app_service.locale(Some(&req), Some(&session), Some(&user));

    let ctx = json!({
        "locale": locale,
        "locales": locales,
        "user" : user,
        "alerts": alerts,
        "dark_mode": dark_mode
    });
    let s = tmpl
        .get_ref()
        .render_throw_http("pages/profile/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
