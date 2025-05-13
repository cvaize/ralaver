use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::{
    AppService, Session, TemplateService, TranslatorService, User, WebAuthService, WebHttpResponse,
};
use actix_web::web::{Data, Form, Query, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use serde_derive::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize, Debug)]
pub struct IndexQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

pub async fn invoke(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    query: Query<IndexQuery>,
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

    let page = query.page.unwrap_or(1);
    let page_str = page.to_string();
    let per_page = query.per_page.unwrap_or(10);
    let per_page_str = per_page.to_string();

    let mut context_data = get_context_data(
        &req,
        user,
        &session,
        translator_service,
        app_service,
        web_auth_service,
    );
    let lang = &context_data.lang;
    let mut page_vars: HashMap<&str, &str> = HashMap::new();
    page_vars.insert("page", &page_str);
    page_vars.insert("per_page", &per_page_str);
    context_data.title = translator_service.variables(lang, "page.users.index.title", &page_vars);

    let layout_ctx = get_template_context(&context_data);

    let ctx = json!({
        "ctx": layout_ctx,
        "heading": translator_service.translate(lang, "page.users.index.header"),
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.users.index.breadcrumbs.home")},
            {"href": "/users", "label": translator_service.translate(lang, "page.users.index.breadcrumbs.users")},
            {"label": translator_service.variables(lang, "page.users.index.breadcrumbs.index", &page_vars)},
        ],
        "create": {
            "href": "/users/create",
            "label": translator_service.translate(lang, "page.users.index.create")
        },
    });
    let s = tmpl_service.render_throw_http("pages/users/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}
