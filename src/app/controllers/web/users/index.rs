use crate::app::controllers::web::{generate_pagination_array, get_context_data, get_template_context};
use crate::{AppService, Config, Session, TemplateService, TranslatorService, User, UserService, WebAuthService, WebHttpResponse};
use actix_web::web::{Data, Form, Query, ReqData};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::cmp::{min, max};
use serde::Deserialize;
use crate::app::repositories::UserPaginateParams;

pub const PER_PAGES: [i64; 6] = [
    10,
    20,
    30,
    40,
    50,
    100
];

#[derive(Deserialize, Debug)]
pub struct IndexQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
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
    user_service: Data<UserService>,
) -> Result<HttpResponse, Error> {
    let translator_service = translator_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let user = user.as_ref();

    let page = max(query.page.unwrap_or(1), 1);
    let page_str = page.to_string();
    let per_page = min(query.per_page.unwrap_or(10), 100);

    let pagination_params = UserPaginateParams::simple(page, per_page);
    let users = user_service.paginate(&pagination_params).map_err(|e| {
        error::ErrorInternalServerError("")
    })?;
    let total_pages_str = users.total_pages.to_string();

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
    page_vars.insert("total_pages", &total_pages_str);
    context_data.title = translator_service.variables(lang, "page.users.index.title", &page_vars);

    let layout_ctx = get_template_context(&context_data);

    let pagination_nums = generate_pagination_array(users.page, users.total_pages);
    let pagination_link = format!("/users?per_page={per_page}&page=");

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
        "page_per_page": translator_service.variables(lang, "page.users.index.page_per_page", &page_vars),
        "per_page_label": translator_service.translate(lang, "page.users.index.per_page_label"),
        "select_page": translator_service.translate(lang, "page.users.index.select_page"),
        "sort": {
            "label": translator_service.translate(lang, "page.users.index.sort")
        },
        "selected": {
            "label": translator_service.translate(lang, "page.users.index.selected.label"),
            "delete": translator_service.translate(lang, "page.users.index.selected.delete"),
            "delete_q": translator_service.translate(lang, "page.users.index.selected.delete_q"),
        },
        "columns": {
            "id": translator_service.translate(lang, "page.users.index.columns.id"),
            "email": translator_service.translate(lang, "page.users.index.columns.email"),
            "surname": translator_service.translate(lang, "page.users.index.columns.surname"),
            "name": translator_service.translate(lang, "page.users.index.columns.name"),
            "patronymic": translator_service.translate(lang, "page.users.index.columns.patronymic"),
            "actions": translator_service.translate(lang, "page.users.index.columns.actions")
        },
        "users": {
            "page": users.page,
            "per_page": users.per_page,
            "total_pages": users.total_pages,
            "total_records": users.total_records,
            "records": users.records,
            "pagination_nums": pagination_nums,
            "pagination_link": pagination_link
        },
        "per_pages": &PER_PAGES,
    });
    let s = tmpl_service.render_throw_http("pages/users/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}
