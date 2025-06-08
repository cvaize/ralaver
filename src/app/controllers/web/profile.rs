use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::{AppService, RoleService, TemplateService, TranslatorService};
use crate::{Session, User, WebAuthService};
use actix_web::web::{Data, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;

const ROUTE_NAME: &'static str = "profile_index";

pub async fn index(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    web_auth_service: Data<WebAuthService>,
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let role_service = role_service.get_ref();
    let user = user.as_ref();

    let mut context_data = get_context_data(
        ROUTE_NAME,
        &req,
        user,
        &session,
        translator_service,
        app_service,
        web_auth_service,
        role_service,
    );
    let lang = &context_data.lang;
    context_data.title = translator_service.translate(lang, "page.profile.title");

    let layout_ctx = get_template_context(&context_data);

    let ctx = json!({
        "ctx": layout_ctx,
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.profile.breadcrumbs.home")},
            {"label": translator_service.translate(lang, "page.profile.breadcrumbs.profile")},
        ],
    });
    let s = tmpl_service.render_throw_http("pages/profile/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}
