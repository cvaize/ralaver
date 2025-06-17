use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::app::validator::rules::length::MinMaxLengthString as MMLS;
use crate::app::validator::rules::required::Required;
use crate::libs::actix_web::types::form::Form;
use crate::{
    prepare_value, Alert, AlertVariant, AppService, Disk, File, FileColumn, FilePolicy,
    FileService, FileServiceError, RateLimitService, Role, RoleService, Session, TemplateService,
    TranslatableError, TranslatorService, User, WebAuthService, WebHttpResponse,
};
use actix_web::web::Path;
use actix_web::web::{Data, ReqData};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

const RL_MAX_ATTEMPTS: u64 = 10;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "files_create_update";

const ROUTE_NAME: &'static str = "files_create_update";

#[derive(Deserialize, Default, Debug)]
pub struct PostData {
    pub _token: Option<String>,
    pub action: Option<String>,
    pub local_path: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
struct ErrorMessages {
    pub form: Vec<String>,
    pub local_path: Vec<String>,
    pub name: Vec<String>,
}

pub async fn create(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    role_service: Data<RoleService>,
    file_service: Data<FileService>,
) -> Result<HttpResponse, Error> {
    let user_roles: Vec<Role> = role_service.get_all_throw_http()?;
    if !FilePolicy::can_create(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    let data = Form(PostData::default());
    invoke(
        None,
        req,
        data,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        role_service,
        file_service,
    )
}

pub async fn store(
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    role_service: Data<RoleService>,
    file_service: Data<FileService>,
) -> Result<HttpResponse, Error> {
    let user_roles = role_service.get_all_throw_http()?;
    if !FilePolicy::can_create(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    invoke(
        None,
        req,
        data,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        role_service,
        file_service,
    )
}

pub async fn edit(
    path: Path<u64>,
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    role_service: Data<RoleService>,
    file_service: Data<FileService>,
) -> Result<HttpResponse, Error> {
    let user_roles = role_service.get_all_throw_http()?;
    if !FilePolicy::can_update(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    let file_id = path.into_inner();
    let edit_file = file_service.get_ref().first_by_id_throw_http(file_id)?;
    let post_data = PostData {
        _token: None,
        action: None,
        name: Some(edit_file.name.to_owned()),
        local_path: Some(edit_file.local_path.to_owned()),
    };
    let edit_file = Some(edit_file);
    let data = Form(post_data);
    invoke(
        edit_file,
        req,
        data,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        role_service,
        file_service,
    )
}

pub async fn update(
    path: Path<u64>,
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    role_service: Data<RoleService>,
    file_service: Data<FileService>,
) -> Result<HttpResponse, Error> {
    let user_roles = role_service.get_all_throw_http()?;
    if !FilePolicy::can_update(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    let file_id = path.into_inner();
    let edit_file = Some(file_service.get_ref().first_by_id_throw_http(file_id)?);
    invoke(
        edit_file,
        req,
        data,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        role_service,
        file_service,
    )
}

pub fn invoke(
    edit_file: Option<File>,
    req: HttpRequest,
    mut data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    role_service: Data<RoleService>,
    file_service: Data<FileService>,
) -> Result<HttpResponse, Error> {
    data.prepare();
    //
    let translator_service = translator_service.get_ref();
    let template_service = template_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let role_service = role_service.get_ref();
    let file_service = file_service.get_ref();

    //
    let user = user.as_ref();

    let mut alert_variants: Vec<AlertVariant> = Vec::new();
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

    let name_str = translator_service.translate(lang, "page.files.create.fields.name");
    let local_path_str = translator_service.translate(lang, "page.files.create.fields.local_path");

    let (title, heading, action) = if let Some(edit_file) = &edit_file {
        let mut vars: HashMap<&str, &str> = HashMap::new();
        let name_ = &edit_file.name;
        vars.insert("name", name_);

        (
            translator_service.variables(lang, "page.files.edit.title", &vars),
            translator_service.variables(lang, "page.files.edit.header", &vars),
            get_edit_url(edit_file.id.to_string().as_str()),
        )
    } else {
        (
            translator_service.translate(lang, "page.files.create.title"),
            translator_service.translate(lang, "page.files.create.header"),
            get_create_url(),
        )
    };

    context_data.title = title;

    //
    let is_post = req.method().eq(&Method::POST);
    let mut is_done = false;
    let mut errors = ErrorMessages::default();

    if is_post {
        web_auth_service.check_csrf_throw_http(&session, &data._token)?;

        let rate_limit_key = rate_limit_service.make_key_from_request_throw_http(&req, RL_KEY)?;

        let executed =
            rate_limit_service.attempt_throw_http(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)?;

        if executed {
            errors.local_path = Required::validated(
                translator_service,
                lang,
                &data.local_path,
                |value| MMLS::validate(translator_service, lang, value, 4, 2048, &local_path_str),
                &local_path_str,
            );

            errors.name = Required::validated(
                translator_service,
                lang,
                &data.name,
                |value| MMLS::validate(translator_service, lang, value, 4, 255, &name_str),
                &name_str,
            );

            if errors.is_empty() {
                let id = if let Some(edit_file) = &edit_file {
                    edit_file.id
                } else {
                    0
                };
                let mut file_data = File::default();
                file_data.id = id;
                file_data.name = data.name.clone().unwrap();
                file_data.local_path = data.local_path.clone().unwrap();

                let columns: Option<Vec<FileColumn>> =
                    Some(vec![FileColumn::Name, FileColumn::LocalPath]);

                let result = file_service.upsert(&mut file_data, &columns);

                if let Err(error) = result {
                    if error.eq(&FileServiceError::DuplicateLocalPath) {
                        errors
                            .local_path
                            .push(error.translate(lang, translator_service));
                    } else {
                        errors.form.push(error.translate(lang, translator_service));
                    }
                } else {
                    is_done = true;
                }
            }
        } else {
            let ttl_message = rate_limit_service.ttl_message_throw_http(
                translator_service,
                lang,
                &rate_limit_key,
            )?;
            errors.form.push(ttl_message)
        }

        if is_done {
            rate_limit_service.clear_throw_http(&rate_limit_key)?;
        }
    }

    //
    for form_error in errors.form {
        context_data.alerts.push(Alert::error(form_error));
    }

    if is_done {
        let mut id: String = "".to_string();

        if let Some(edit_file) = &edit_file {
            let file = file_service.first_by_id_throw_http(edit_file.id)?;
            id = file.id.to_string();
            let name_ = file.name;
            alert_variants.push(AlertVariant::FilesUpdateSuccess(name_))
        } else if let Some(local_path_) = &data.local_path {
            let file = file_service.first_by_local_path_throw_http(&Disk::Local, local_path_)?;
            id = file.id.to_string();
            let name_ = file.name;
            alert_variants.push(AlertVariant::FilesCreateSuccess(name_))
        }

        if let Some(action) = &data.action {
            if action.eq("save") {
                let url_ = get_edit_url(&id);
                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((
                        http::header::LOCATION,
                        http::HeaderValue::from_str(&url_)
                            .map_err(|_| error::ErrorInternalServerError(""))?,
                    ))
                    .finish());
            } else if action.eq("save_and_close") {
                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((
                        http::header::LOCATION,
                        http::HeaderValue::from_static("/files"),
                    ))
                    .finish());
            }
        }
    }

    for variant in &alert_variants {
        context_data
            .alerts
            .push(Alert::from_variant(translator_service, lang, variant));
    }

    let layout_ctx = get_template_context(&context_data);

    let fields = json!({
        "name": { "label": name_str, "value": &data.name, "errors": errors.name },
        "local_path": { "label": local_path_str, "value": &data.local_path, "errors": errors.local_path },
    });

    let ctx = json!({
        "ctx": layout_ctx,
        "heading": &heading,
        "tabs": {
            "main": translator_service.translate(lang, "page.files.create.tabs.main")
        },
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
            {"href": "/files", "label": translator_service.translate(lang, "page.files.index.header")},
            {"label": &heading},
        ],
        "form": {
            "action": &action,
            "method": "post",
            "fields": fields,
            "save": translator_service.translate(lang, "Save"),
            "save_and_close": translator_service.translate(lang, "Save and close"),
            "close": {
                "label": translator_service.translate(lang, "Close"),
                "href": "/files"
            },
        },
    });
    let s = template_service.render_throw_http("pages/files/create-update.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

pub fn get_create_url() -> String {
    "/files/create".to_string()
}

pub fn get_edit_url(id: &str) -> String {
    let mut str_ = "/files/".to_string();
    str_.push_str(id);
    str_
}

impl PostData {
    pub fn prepare(&mut self) {
        prepare_value!(self._token);
        prepare_value!(self.action);
        prepare_value!(self.name);
        prepare_value!(self.local_path);
    }
}

impl ErrorMessages {
    pub fn is_empty(&self) -> bool {
        self.form.len() == 0 && self.name.len() == 0 && self.local_path.len() == 0
    }
}
