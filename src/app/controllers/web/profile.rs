use crate::Session;
use crate::{
    Alert, AppService, KeyValueService, SessionService, TemplateService, User, ALERTS_KEY,
};
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use serde_json::json;

pub async fn index(
    req: HttpRequest,
    session: Session,
    user: User,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    session_service: Data<SessionService>,
    key_value_service: Data<KeyValueService>,
) -> Result<HttpResponse, Error> {
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let session_service = session_service.get_ref();
    let key_value_service = key_value_service.get_ref();

    let key = session_service.make_session_data_key(&session, ALERTS_KEY);
    let alerts: Vec<Alert> = key_value_service
        .get_del(&key)
        .map_err(|_| error::ErrorInternalServerError("KeyValueService error"))?
        .unwrap_or(vec![]);

    let dark_mode = app_service.dark_mode(&req);

    let (_, locale, locales) = app_service.locale(Some(&req), Some(&session), Some(&user));

    let ctx = json!({
        "locale": locale,
        "locales": locales,
        "user" : user,
        "alerts": alerts,
        "dark_mode": dark_mode
    });
    let s = tmpl_service.render_throw_http("pages/profile/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
