use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::{
    AppService, Session, TemplateService, TranslatorService, User, WebAuthService, WebHttpResponse,
};
use actix_web::web::{Data, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;

const ROUTE_NAME: &'static str = "home_index";

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

    let mut context_data = get_context_data(
        ROUTE_NAME,
        &req,
        user,
        &session,
        translator_service,
        app_service,
        web_auth_service,
    );
    let lang = &context_data.lang;
    context_data.title = translator_service.translate(lang, "page.home.title");

    let layout_ctx = get_template_context(&context_data);

    let ctx = json!({
        "ctx": layout_ctx,
        "breadcrumbs": [
            {"label": translator_service.translate(lang, "page.home.breadcrumbs.home")}
        ],
    });
    let s = tmpl_service.render_throw_http("pages/home/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}
