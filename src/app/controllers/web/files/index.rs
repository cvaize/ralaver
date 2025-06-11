use crate::app::controllers::web::{
    generate_2_offset_pagination_array, get_context_data, get_template_context,
};
use crate::{FilePolicy, prepare_paginate, prepare_value, validation_query_max_length_string, Alert, AppService, LocaleService, Role, RoleFilter, RolePaginateParams, RoleService, RoleSort, Session, TemplateService, TranslatorService, User, WebAuthService, WebHttpResponse, FileService, FilePaginateParams, FileFilter, FileSort};
use actix_web::web::{Data, Query, ReqData};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::max;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use strum::IntoEnumIterator;

const PAGE_URL: &'static str = "/files?";

pub const DEFAULT_PER_PAGE: i64 = 15;
pub const MAX_PER_PAGE: i64 = 100;
pub const PER_PAGES: [i64; 7] = [10, 15, 20, 30, 40, 50, 100];

const ROUTE_NAME: &'static str = "files_index";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
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
    role_service: Data<RoleService>,
    file_service: Data<FileService>,
    locale_service: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    let tr_s = translator_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let locale_service = locale_service.get_ref();
    let role_service = role_service.get_ref();
    let file_service = file_service.get_ref();
    let user = user.as_ref();

    let user_roles = role_service.get_all_throw_http()?;
    if !FilePolicy::can_show(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }

    query.prepare();

    let lang: String = locale_service.get_locale_code(Some(&req), Some(&user));
    let lang = &lang;

    let search_str = tr_s.translate(lang, "Search");
    let reset_str = tr_s.translate(lang, "Reset");
    let sort_str = tr_s.translate(lang, "Sort");

    let form_errors: Vec<String> = query.validate(tr_s, lang, &search_str, &sort_str);

    let page = query.page.unwrap();
    let per_page = query.per_page.unwrap();
    let page_str = page.to_string();
    let filters: Vec<FileFilter> = query.get_filters();
    let sort = query.get_sort();
    let pagination_params = FilePaginateParams::new(page, per_page, filters, sort);
    let files = file_service.paginate_throw_http(&pagination_params)?;
    let total_pages = max(files.total_pages, 1);
    let total_pages_str = total_pages.to_string();

    let mut context_data = get_context_data(
        ROUTE_NAME,
        &req,
        user,
        &session,
        tr_s,
        app_service,
        web_auth_service,
        role_service,
    );
    let mut page_vars: HashMap<&str, &str> = HashMap::new();
    page_vars.insert("page", &page_str);
    page_vars.insert("total_pages", &total_pages_str);
    context_data.title = tr_s.variables(lang, "page.files.index.title", &page_vars);

    for form_error in form_errors {
        context_data.alerts.push(Alert::error(form_error));
    }

    let layout_ctx = get_template_context(&context_data);

    let mut pagination_link = query.clone().remove_page().to_url()?;
    pagination_link.push_str("&page=:page");
    let pagination_nums = generate_2_offset_pagination_array(files.page, total_pages);

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

    let mut sort_options: Vec<Value> = Vec::new();
    for sort_enum in FileSort::iter() {
        let value = sort_enum.to_string();
        let mut key = "page.files.index.sort.".to_string();
        key.push_str(&value);
        let label = tr_s.translate(lang, &key);
        let value = sort_enum.to_string();
        sort_options.push(json!({ "label": label, "value": value }));
    }

    let mut selected: Option<Value> = None;
    let mut create: Option<Value> = None;
    let mut edit: Option<Value> = None;
    let mut delete: Option<Value> = None;

    if FilePolicy::can_create(&user, &user_roles) {
        create = Some(json!({
            "label": tr_s.translate(lang, "Create file"),
            "href": "/files/create"
        }));
    }

    if FilePolicy::can_update(&user, &user_roles) {
        edit = Some(json!({
            "label": tr_s.translate(lang, "Edit file"),
            "href": "/files/:id"
        }));
    }

    if FilePolicy::can_delete(&user, &user_roles) {
        selected = Some(json!({
            "label": tr_s.translate(lang, "Selected"),
            "delete": tr_s.translate(lang, "Delete selected"),
            "delete_confirm": tr_s.translate(lang, "Delete selected?"),
        }));
        delete = Some(json!({
            "action": "/files/:id/delete",
            "method": "post",
            "label": tr_s.translate(lang, "Delete file"),
            "confirm": tr_s.translate(lang, "Delete file(ID: :id)?"),
        }));
    }

    let ctx = json!({
        "ctx": &layout_ctx,
        "heading": tr_s.translate(lang, "page.files.index.header"),
        "breadcrumbs": [
            {"href": "/", "label": tr_s.translate(lang, "page.home.header")},
            {"href": "/files", "label": tr_s.translate(lang, "page.files.index.header")},
            {"label": tr_s.variables(lang, "Page :page of :total_pages", &page_vars)},
        ],
        "create": create,
        "edit": edit,
        "delete": delete,
        "page_per_page": tr_s.variables(lang, "Page :page of :total_pages", &page_vars),
        "per_page_label": tr_s.translate(lang, "Number of entries per page"),
        "select_page": tr_s.translate(lang, "Select page"),
        "sort": {
            "label": &sort_str,
            "value": &query.sort,
            "options": &sort_options
        },
        "selected": selected,
        "columns": {
            "id": tr_s.translate(lang, "page.files.index.columns.id"),
            "name": tr_s.translate(lang, "page.files.index.columns.name"),
            "url": tr_s.translate(lang, "page.files.index.columns.url"),
            "is_deleted": tr_s.translate(lang, "page.files.index.columns.is_deleted"),
            "actions": tr_s.translate(lang, "page.files.index.columns.actions")
        },
        "files": {
            "page": files.page,
            "per_page": files.per_page,
            "total_pages": total_pages,
            "total_records": files.total_records,
            "records": files.records,
            "pagination_nums": pagination_nums,
            "pagination_link": pagination_link
        },
        "per_pages": &PER_PAGES,
        "filter_label": tr_s.translate(lang, "Filters"),
        "close_label": tr_s.translate(lang, "Close"),
        "apply_label": tr_s.translate(lang, "Apply"),
        "mass_actions": {
            "action": "/files",
            "method": "post",
        },
        "filter": {
            "search": {
                "label": search_str,
                "values": search_values,
                "value": &query.search,
                "action": "/files",
                "method": "get",
                "reset": {
                    "href": &link_without_search,
                    "label": &reset_str
                }
            }
        }
    });

    let s = tmpl_service.render_throw_http("pages/files/index.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

impl IndexQuery {
    pub fn prepare(&mut self) {
        prepare_paginate!(self.page, self.per_page, DEFAULT_PER_PAGE, MAX_PER_PAGE);
        prepare_value!(self.search);
        prepare_value!(self.sort);
        if self.sort.is_none() {
            self.sort = Some(FileSort::IdDesc.to_string());
        }
    }
    pub fn validate(
        &mut self,
        t_s: &TranslatorService,
        lang: &str,
        search_str: &str,
        sort_str: &str,
    ) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();

        validation_query_max_length_string!(errors, self.search, search_str, 255, t_s, lang);
        validation_query_max_length_string!(errors, self.sort, sort_str, 255, t_s, lang);

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
    pub fn remove_sort(&mut self) -> &mut Self {
        self.sort = None;
        self
    }
    pub fn to_url(&self) -> Result<String, Error> {
        let url = serde_urlencoded::to_string(self).map_err(|e| {
            log::error!("app::controllers::web::files::index::IndexQuery::to_url - {e}");
            error::ErrorInternalServerError("")
        })?;
        let mut result = PAGE_URL.to_string();
        result.push_str(&url);
        Ok(result)
    }
    pub fn get_filters(&self) -> Vec<FileFilter> {
        let mut filters: Vec<FileFilter> = Vec::new();

        if let Some(value) = &self.search {
            filters.push(FileFilter::Search(value));
        }
        filters
    }
    pub fn get_sort(&self) -> Option<FileSort> {
        let mut sort = None;
        if let Some(sort_) = &self.sort {
            if let Ok(sort__) = FileSort::from_str(sort_) {
                sort = Some(sort__);
            }
        }
        sort
    }
}
