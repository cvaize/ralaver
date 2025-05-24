use crate::app::controllers::web::{
    generate_pagination_array, get_context_data, get_template_context,
};
use crate::app::repositories::{UserFilter, UserPaginateParams, UserSort};
use crate::app::validator::rules::length::MaxLengthString;
use crate::{
    Alert, AppService, Config, Locale, LocaleService, Session, TemplateService, TranslatorService,
    User, UserService, WebAuthService, WebHttpResponse,
};
use actix_web::web::{Data, Form, Query, ReqData};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::{max, min};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use strum::{EnumMessage, IntoEnumIterator};

static PAGE_URL: &str = "/users?";

pub const PER_PAGES: [i64; 6] = [10, 20, 30, 40, 50, 100];

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
) -> Result<HttpResponse, Error> {
    let translator_service = translator_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let locale_service = locale_service.get_ref();
    let user = user.as_ref();

    let lang: String = locale_service.get_locale_code(Some(&req), Some(&user));
    let lang = &lang;

    let search_str = translator_service.translate(lang, "Search");
    let reset_str = translator_service.translate(lang, "Reset");
    let locale_str = translator_service.translate(lang, "page.users.index.columns.locale");
    let sort_str = translator_service.translate(lang, "Sort");

    let mut form_errors: Vec<String> = query.prepare(
        translator_service,
        lang,
        &search_str,
        &locale_str,
        &sort_str,
    );

    let page = query.page.unwrap();
    let per_page = query.per_page.unwrap();

    let page_str = page.to_string();

    let mut filters: Vec<UserFilter> = Vec::new();

    if let Some(value) = &query.search {
        filters.push(UserFilter::Search(value));
    }
    if let Some(value) = &query.locale {
        filters.push(UserFilter::Locale(value));
    }

    let mut sort = None;
    if let Some(sort_) = &query.sort {
        if let Ok(sort__) = UserSort::from_str(sort_) {
            sort = Some(sort__);
        }
    }
    let pagination_params = UserPaginateParams::new(page, per_page, filters, sort);
    let users = user_service
        .paginate(&pagination_params)
        .map_err(|e| error::ErrorInternalServerError(""))?;
    let total_pages = if users.total_pages <= 0 {
        1
    } else {
        users.total_pages
    };
    let total_pages_str = total_pages.to_string();

    let mut context_data = get_context_data(
        &req,
        user,
        &session,
        translator_service,
        app_service,
        web_auth_service,
    );
    let mut page_vars: HashMap<&str, &str> = HashMap::new();
    page_vars.insert("page", &page_str);
    page_vars.insert("total_pages", &total_pages_str);
    context_data.title = translator_service.variables(lang, "page.users.index.title", &page_vars);

    for form_error in form_errors {
        context_data.alerts.push(Alert::error(form_error));
    }

    let layout_ctx = get_template_context(&context_data);

    let mut pagination_link = query.without_search().to_url()?;
    pagination_link.push_str("&page=");
    let pagination_nums = generate_pagination_array(users.page, total_pages);

    let link_without_search = query.without_search().to_url()?;
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

    let link_without_locale = query.without_locale().to_url()?;
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
        let mut key = String::from("page.users.index.sort.");
        key.push_str(&value);
        let label = translator_service.translate(lang, &key);
        let value = sort_enum.to_string();
        sort_options.push(json!({ "label": label, "value": value }));
    }

    let ctx = json!({
        "ctx": &layout_ctx,
        "heading": translator_service.translate(lang, "page.users.index.header"),
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
            {"href": "/users", "label": translator_service.translate(lang, "page.users.index.header")},
            {"label": translator_service.variables(lang, "Page :page of :total_pages", &page_vars)},
        ],
        "create": {
            "href": "/users/create",
            "label": translator_service.translate(lang, "page.users.index.create")
        },
        "page_per_page": translator_service.variables(lang, "Page :page of :total_pages", &page_vars),
        "per_page_label": translator_service.translate(lang, "Number of entries per page"),
        "select_page": translator_service.translate(lang, "Select page"),
        "sort": {
            "label": &sort_str,
            "value": &query.sort,
            "options": &sort_options
        },
        "selected": {
            "label": translator_service.translate(lang, "Selected"),
            "delete": translator_service.translate(lang, "Delete selected"),
            "delete_q": translator_service.translate(lang, "Delete selected?"),
        },
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
        "filter": {
            "search": {
                "label": search_str,
                "values": search_values,
                "value": &query.search,
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
    pub fn prepare(
        &mut self,
        translator_service: &TranslatorService,
        lang: &str,
        search_str: &str,
        locale_str: &str,
        sort_str: &str,
    ) -> Vec<String> {
        let query = self;
        let page = max(query.page.unwrap_or(1), 1);
        let per_page = min(query.per_page.unwrap_or(10), 100);
        query.page = Some(page);
        query.per_page = Some(per_page);

        let mut errors: Vec<String> = Vec::new();
        if let Some(value_original) = &query.search {
            let value = value_original.trim();
            if value.len() == 0 {
                query.search = None;
            } else {
                let mut errors_: Vec<String> =
                    MaxLengthString::validate(translator_service, lang, value, 255, search_str);
                if errors_.len() != 0 {
                    errors.append(&mut errors_);
                    query.search = None;
                } else if value.len() != value_original.len() {
                    query.search = Some(value.to_owned());
                }
            }
        }
        if let Some(value_original) = &query.locale {
            let value = value_original.trim();
            if value.len() == 0 {
                query.locale = None;
            } else {
                let mut errors_: Vec<String> =
                    MaxLengthString::validate(translator_service, lang, value, 6, locale_str);
                if errors_.len() != 0 {
                    errors.append(&mut errors_);
                    query.locale = None;
                } else if value.len() != value_original.len() {
                    query.locale = Some(value.to_owned());
                }
            }
        }
        if let Some(value_original) = &query.sort {
            let value = value_original.trim();
            if value.len() == 0 {
                query.sort = None;
            } else {
                let mut errors_: Vec<String> =
                    MaxLengthString::validate(translator_service, lang, value, 255, sort_str);
                if errors_.len() != 0 {
                    errors.append(&mut errors_);
                    query.sort = None;
                } else if value.len() != value_original.len() {
                    query.sort = Some(value.to_owned());
                }
            }
        }

        if query.sort.is_none() {
            query.sort = Some(UserSort::IdDesc.to_string());
        }

        errors
    }
    pub fn without_page(&self) -> Self {
        let mut query = self.clone();
        query.page = None;
        query
    }
    pub fn without_per_page(&self) -> Self {
        let mut query = self.clone();
        query.per_page = None;
        query
    }
    pub fn without_search(&self) -> Self {
        let mut query = self.clone();
        query.search = None;
        query
    }
    pub fn without_locale(&self) -> Self {
        let mut query = self.clone();
        query.locale = None;
        query
    }
    pub fn without_sort(&self) -> Self {
        let mut query = self.clone();
        query.sort = None;
        query
    }
    pub fn to_url(&self) -> Result<String, Error> {
        let url = serde_urlencoded::to_string(self).map_err(|e| {
            log::error!("app::controllers::web::users::index::IndexQuery::to_url - {e}");
            error::ErrorInternalServerError("")
        })?;
        let mut result = String::from(PAGE_URL);
        result.push_str(&url);
        Ok(result)
    }
}
