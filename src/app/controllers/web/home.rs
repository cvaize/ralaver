use crate::{
    AppService, Translator, TranslatorService, WebHttpRequest, WebHttpResponse,
};
use crate::{AuthService, TemplateService};
use actix_web::web::Data;
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use serde_json::json;

pub async fn index(
    req: HttpRequest,
    translator_service: Data<TranslatorService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    auth_service: Data<AuthService<'_>>,
) -> Result<HttpResponse, Error> {
    let translator_service = translator_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let auth_service = auth_service.get_ref();

    let (user, auth_token) = auth_service
        .login_by_req(&req)
        .map_err(|_| error::ErrorUnauthorized("Unauthorized"))?;

    let dark_mode = app_service.dark_mode(&req);
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&user));
    let translator = Translator::new(&lang, translator_service);

    let ctx = json!({
        "locale": locale,
        "locales": locales,
        "user" : user,
        "alerts": req.get_alerts(&translator),
        "dark_mode": dark_mode
    });
    let s = tmpl_service.render_throw_http("pages/home/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .cookie(auth_service.make_auth_token_cookie(&auth_token))
        .clear_alerts()
        .content_type("text/html")
        .body(s))
}
