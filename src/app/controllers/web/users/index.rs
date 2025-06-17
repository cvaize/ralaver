use crate::app::controllers::web::{
    generate_2_offset_pagination_array, get_context_data, get_template_context,
};
use crate::{
    prepare_paginate, prepare_value, validation_query_max_length_string, Alert, AppService,
    LocaleService, RoleService, Session, TemplateService, TranslatorService, User, UserFilter,
    UserPaginateParams, UserPolicy, UserService, UserSort, WebAuthService, WebHttpResponse,
};
use actix_web::web::{Data, Query, ReqData};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::max;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use strum::IntoEnumIterator;

const PAGE_URL: &'static str = "/users?";

pub const DEFAULT_PER_PAGE: i64 = 15;
pub const MAX_PER_PAGE: i64 = 100;
pub const PER_PAGES: [i64; 7] = [10, 15, 20, 30, 40, 50, 100];

const ROUTE_NAME: &'static str = "users_index";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub locale: Option<String>,
    pub sort: Option<String>,
}

pub async fn invoke(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    mut query: Query<IndexQuery>,
    translator_service: Data<TranslatorService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
    role_service: Data<RoleService>,
) -> Result<HttpResponse, Error> {
    let translator_service = translator_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let locale_service = locale_service.get_ref();
    let user_service = user_service.get_ref();
    let role_service = role_service.get_ref();
    let user = user.as_ref();

    let user_roles = role_service.get_all_throw_http()?;
    if !UserPolicy::can_show(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }

    query.prepare();

    let lang: String = locale_service.get_locale_code(Some(&req), Some(&user));
    let lang = &lang;

    let search_str = translator_service.translate(lang, "Search");
    let reset_str = translator_service.translate(lang, "Reset");
    let locale_str = translator_service.translate(lang, "page.users.index.columns.locale");
    let sort_str = translator_service.translate(lang, "Sort");

    let form_errors: Vec<String> = query.validate(
        translator_service,
        lang,
        &search_str,
        &locale_str,
        &sort_str,
    );

    let page = query.page.unwrap();
    let per_page = query.per_page.unwrap();
    let page_str = page.to_string();
    let filters: Vec<UserFilter> = query.get_filters();
    let sorts: Vec<UserSort> = query.get_sorts();
    let pagination_params = UserPaginateParams::new(page, per_page, filters, sorts);
    let users = user_service.paginate_throw_http(&pagination_params)?;
    let total_pages = max(users.total_pages, 1);
    let total_pages_str = total_pages.to_string();

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
    let mut page_vars: HashMap<&str, &str> = HashMap::new();
    page_vars.insert("page", &page_str);
    page_vars.insert("total_pages", &total_pages_str);
    context_data.title = translator_service.variables(lang, "page.users.index.title", &page_vars);

    for form_error in form_errors {
        context_data.alerts.push(Alert::error(form_error));
    }

    let layout_ctx = get_template_context(&context_data);

    let mut pagination_link = query.clone().remove_page().to_url()?;
    pagination_link.push_str("&page=:page");
    let pagination_nums = generate_2_offset_pagination_array(users.page, total_pages);

    let link_without_search = query.clone().remove_page().remove_search().to_url()?;
    let mut search_values = Vec::new();
    if let Some(search) = &query.search {
        search_values.push(json!({
            "value": search,
            "label": search,
            "reset": {
                "href": &link_without_search,
                "label": &reset_str
            }
        }));
    }

    let link_without_locale = query.clone().remove_page().remove_locale().to_url()?;
    let mut locale_values = Vec::new();
    if let Some(locale) = &query.locale {
        locale_values.push(json!({
            "value": locale,
            "label": locale,
            "reset": {
                "href": &link_without_locale,
                "label": &reset_str
            }
        }));
    }

    let mut sort_options: Vec<Value> = Vec::new();
    for sort_enum in UserSort::iter() {
        let value = sort_enum.to_string();
        let mut key = "page.users.index.sort.".to_string();
        key.push_str(&value);
        let label = translator_service.translate(lang, &key);
        let value = sort_enum.to_string();
        sort_options.push(json!({ "label": label, "value": value }));
    }

    let mut selected: Option<Value> = None;
    let mut create: Option<Value> = None;
    let mut edit: Option<Value> = None;
    let mut delete: Option<Value> = None;

    if UserPolicy::can_create(&user, &user_roles) {
        create = Some(json!({
            "label": translator_service.translate(lang, "Create user"),
            "href": "/users/create"
        }));
    }

    if UserPolicy::can_update(&user, &user_roles) {
        edit = Some(json!({
            "label": translator_service.translate(lang, "Edit user"),
            "href": "/users/:id"
        }));
    }

    if UserPolicy::can_delete(&user, &user_roles) {
        selected = Some(json!({
            "label": translator_service.translate(lang, "Selected"),
            "delete": translator_service.translate(lang, "Delete selected"),
            "delete_confirm": translator_service.translate(lang, "Delete selected?"),
        }));
        delete = Some(json!({
            "action": "/users/:id/delete",
            "method": "post",
            "label": translator_service.translate(lang, "Delete user"),
            "confirm": translator_service.translate(lang, "Delete user(ID: :id)?"),
        }));
    }

    let ctx = json!({
        "ctx": &layout_ctx,
        "heading": translator_service.translate(lang, "page.users.index.header"),
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
            {"href": "/users", "label": translator_service.translate(lang, "page.users.index.header")},
            {"label": translator_service.variables(lang, "Page :page of :total_pages", &page_vars)},
        ],
        "create": create,
        "edit": edit,
        "delete": delete,
        "page_per_page": translator_service.variables(lang, "Page :page of :total_pages", &page_vars),
        "per_page_label": translator_service.translate(lang, "Number of entries per page"),
        "select_page": translator_service.translate(lang, "Select page"),
        "sort": {
            "label": &sort_str,
            "value": &query.sort,
            "options": &sort_options
        },
        "selected": selected,
        "columns": {
            "id": translator_service.translate(lang, "page.users.index.columns.id"),
            "email": translator_service.translate(lang, "page.users.index.columns.email"),
            "surname": translator_service.translate(lang, "page.users.index.columns.surname"),
            "name": translator_service.translate(lang, "page.users.index.columns.name"),
            "patronymic": translator_service.translate(lang, "page.users.index.columns.patronymic"),
            "locale": locale_str,
            "actions": translator_service.translate(lang, "page.users.index.columns.actions")
        },
        "users": {
            "page": users.page,
            "per_page": users.per_page,
            "total_pages": total_pages,
            "total_records": users.total_records,
            "records": users.records,
            "pagination_nums": pagination_nums,
            "pagination_link": pagination_link
        },
        "per_pages": &PER_PAGES,
        "filter_label": translator_service.translate(lang, "Filters"),
        "close_label": translator_service.translate(lang, "Close"),
        "apply_label": translator_service.translate(lang, "Apply"),
        "mass_actions": {
            "action": "/users",
            "method": "post",
        },
        "filter": {
            "search": {
                "label": search_str,
                "values": search_values,
                "value": &query.search,
                "action": "/users",
                "method": "get",
                "reset": {
                    "href": &link_without_search,
                    "label": &reset_str
                }
            },
            "locale": {
                "label": locale_str,
                "values": locale_values,
                "value": &query.locale,
                "placeholder": translator_service.translate(lang, "Not selected..."),
                "options": layout_ctx.get("locales"),
                "reset": {
                    "href": &link_without_locale,
                    "label": &reset_str
                }
            }
        }
    });

    let s = tmpl_service.render_throw_http("pages/users/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

impl IndexQuery {
    pub fn prepare(&mut self) {
        prepare_paginate!(self.page, self.per_page, DEFAULT_PER_PAGE, MAX_PER_PAGE);
        prepare_value!(self.search);
        prepare_value!(self.locale);
        prepare_value!(self.sort);
        if self.sort.is_none() {
            self.sort = Some(UserSort::IdDesc.to_string());
        }
    }
    pub fn validate(
        &mut self,
        translator_service: &TranslatorService,
        lang: &str,
        search_str: &str,
        locale_str: &str,
        sort_str: &str,
    ) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();

        validation_query_max_length_string!(
            errors,
            self.search,
            search_str,
            255,
            translator_service,
            lang
        );
        validation_query_max_length_string!(
            errors,
            self.locale,
            locale_str,
            6,
            translator_service,
            lang
        );
        validation_query_max_length_string!(
            errors,
            self.sort,
            sort_str,
            255,
            translator_service,
            lang
        );

        errors
    }
    pub fn remove_page(&mut self) -> &mut Self {
        self.page = None;
        self
    }
    pub fn remove_per_page(&mut self) -> &mut Self {
        self.per_page = None;
        self
    }
    pub fn remove_search(&mut self) -> &mut Self {
        self.search = None;
        self
    }
    pub fn remove_locale(&mut self) -> &mut Self {
        self.locale = None;
        self
    }
    pub fn remove_sort(&mut self) -> &mut Self {
        self.sort = None;
        self
    }
    pub fn to_url(&self) -> Result<String, Error> {
        let url = serde_urlencoded::to_string(self).map_err(|e| {
            log::error!("app::controllers::web::users::index::IndexQuery::to_url - {e}");
            error::ErrorInternalServerError("")
        })?;
        let mut result = PAGE_URL.to_string();
        result.push_str(&url);
        Ok(result)
    }
    pub fn get_filters(&self) -> Vec<UserFilter> {
        let mut filters: Vec<UserFilter> = Vec::new();

        if let Some(value) = &self.search {
            filters.push(UserFilter::Search(value.to_string()));
        }
        if let Some(value) = &self.locale {
            filters.push(UserFilter::Locale(value.to_string()));
        }
        filters
    }
    pub fn get_sorts(&self) -> Vec<UserSort> {
        let mut sorts: Vec<UserSort> = Vec::new();
        if let Some(sort_) = &self.sort {
            if let Ok(sort__) = UserSort::from_str(sort_) {
                sorts.push(sort__);
            }
        }
        sorts
    }
}
