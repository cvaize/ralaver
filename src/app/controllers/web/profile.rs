use actix_web::web::Data;
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use crate::{AlertService, AppService, TemplateService, User};
use actix_session::Session;
use serde_json::json;

pub async fn index(
    req: HttpRequest,
    tmpl: Data<TemplateService>,
    alert_service: Data<AlertService>,
    app_service: Data<AppService>,
    session: Session,
    user: User,
) -> Result<HttpResponse, Error> {
    let alerts = alert_service
        .get_ref()
        .get_and_remove_from_session(&session)
        .unwrap_or(Vec::new());

    let dark_mode = app_service.get_ref().get_dark_mode(&req);

    let lang = app_service.get_locale_code(Some(&req), Some(&session), Some(&user));
    let locale = app_service.get_locale_or_default_ref(&lang);
    let locales = app_service.get_locales_or_default_without_current_ref(&locale.code);

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
