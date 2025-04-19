use crate::{Alert, AppService, Session, User, ALERTS_KEY};
use crate::{FlashService, TemplateService};
use actix_web::web::Data;
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use serde_json::json;

pub async fn index(
    req: HttpRequest,
    session: Session,
    user: User,
    flash_service: Data<FlashService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
) -> Result<HttpResponse, Error> {
    let flash_service = flash_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();

    let alerts: Vec<Alert> = flash_service.all_throw_http(&session, ALERTS_KEY)?.unwrap_or(vec![]);
    let dark_mode = app_service.dark_mode(&req);
    let (_, locale, locales) = app_service.locale(Some(&req), Some(&session), Some(&user));

    let ctx = json!({
        "locale": locale,
        "locales": locales,
        "user" : user,
        "alerts": alerts,
        "dark_mode": dark_mode
    });
    let s = tmpl_service.render_throw_http("pages/home/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
