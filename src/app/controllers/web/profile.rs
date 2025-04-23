use crate::User;
use crate::{AppService, TemplateService, Translator, TranslatorService, WebHttpRequest};
use actix_web::web::{Data, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::rc::Rc;

pub async fn index(
    req: HttpRequest,
    user: ReqData<Rc<User>>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let user = user.as_ref();

    let dark_mode = app_service.dark_mode(&req);

    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&user));
    let translator = Translator::new(&lang, translator_service);

    let ctx = json!({
        "locale": locale,
        "locales": locales,
        "user" : &user,
        "alerts": req.get_alerts(&translator),
        "dark_mode": dark_mode
    });
    let s = tmpl_service.render_throw_http("pages/profile/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
