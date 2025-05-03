use crate::{
    AppService, Session, TemplateService, TranslatorService, User, WebAuthService,
    WebHttpRequest, WebHttpResponse,
};
use actix_web::web::{Data, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;

pub async fn index(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
) -> Result<HttpResponse, Error> {
    let translator_service = translator_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let user = user.as_ref();

    let dark_mode = app_service.dark_mode(&req);
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(user));

    let csrf = web_auth_service.new_csrf(&session);
    let ctx = json!({
        "locale": locale,
        "locales": locales,
        "user" : user,
        "alerts": req.get_alerts(&translator_service, &lang),
        "dark_mode": dark_mode,
        "csrf": csrf
    });
    let s = tmpl_service.render_throw_http("pages/home/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type("text/html")
        .body(s))
}
