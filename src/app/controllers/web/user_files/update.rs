use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::libs::actix_web::types::form::Form;
use crate::{
    prepare_value, Alert, AlertVariant, AppService, File, FilePolicy, FileService,
    RateLimitService, Role, RoleService, Session, TemplateService, TranslatableError,
    TranslatorService, User, UserFile, UserFileColumn, UserFileService, WebAuthService,
    WebHttpResponse,
};
use actix_web::http::header::HeaderValue;
use actix_web::web::Path;
use actix_web::{
    error,
    http::{header::LOCATION, Method},
    web::{Data, ReqData},
    Error, HttpRequest, HttpResponse, Result,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize, Default, Debug)]
pub struct PostData {
    pub _token: Option<String>,
    pub action: Option<String>,
    pub is_public: Option<String>,
}

const RL_MAX_ATTEMPTS: u64 = 10;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "user_files_update";

#[derive(Deserialize, Default, Debug)]
struct ErrorMessages {
    pub form: Vec<String>,
    pub is_public: Vec<String>,
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
    user_file_service: Data<UserFileService>,
    file_service: Data<FileService>,
) -> Result<HttpResponse, Error> {
    let user_roles: Vec<Role> = role_service.all_throw_http()?;
    if !FilePolicy::can_update(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    let id = path.into_inner();
    let edit_user_file = user_file_service.get_ref().first_by_id_throw_http(id)?;
    let edit_file = file_service
        .get_ref()
        .first_by_id_throw_http(edit_user_file.file_id)?;
    let post_data = PostData {
        _token: None,
        action: None,
        is_public: if edit_user_file.is_public {
            Some("1".to_string())
        } else {
            None
        },
    };
    let data = Form(post_data);
    invoke(
        data,
        edit_user_file,
        edit_file,
        req,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        role_service,
        user_file_service,
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
    user_file_service: Data<UserFileService>,
    file_service: Data<FileService>,
) -> Result<HttpResponse, Error> {
    let user_roles = role_service.all_throw_http()?;
    if !FilePolicy::can_update(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    let id = path.into_inner();
    let edit_user_file = user_file_service.get_ref().first_by_id_throw_http(id)?;
    let edit_file = file_service
        .get_ref()
        .first_by_id_throw_http(edit_user_file.file_id)?;
    invoke(
        data,
        edit_user_file,
        edit_file,
        req,
        user,
        session,
        translator_service,
        template_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        role_service,
        user_file_service,
    )
}

pub fn invoke(
    mut data: Form<PostData>,
    mut user_file: UserFile,
    file: File,
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    template_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    role_service: Data<RoleService>,
    user_file_service: Data<UserFileService>,
) -> Result<HttpResponse, Error> {
    data.prepare();
    //
    let translator_service = translator_service.get_ref();
    let template_service = template_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let role_service = role_service.get_ref();

    //
    let user = user.as_ref();

    let mut alert_variants: Vec<AlertVariant> = Vec::new();
    let mut context_data = get_context_data(
        &req,
        user,
        &session,
        translator_service,
        app_service,
        web_auth_service,
        role_service,
    );

    let lang = &context_data.lang;

    let is_public_str = translator_service.translate(lang, "page.files.create.fields.is_public");

    let name = format!("UserFileID:{}", user_file.id);
    let mut vars: HashMap<&str, &str> = HashMap::new();
    vars.insert("name", &name);

    let title = translator_service.variables(lang, "page.files.edit.title", &vars);
    let heading = translator_service.variables(lang, "page.files.edit.header", &vars);

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
            if let Some(value) = &data.is_public {
                user_file.is_public = value.eq("1");
            } else {
                user_file.is_public = false;
            }

            if errors.is_empty() {
                let columns = Some(vec![UserFileColumn::IsPublic]);
                let result = user_file_service.update(user_file.clone(), &columns, &file);

                if let Err(error) = result {
                    errors.form.push(error.translate(lang, translator_service));
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

    let url = get_edit_url(user_file.id.to_string().as_str());
    if is_done {
        alert_variants.push(AlertVariant::RolesUpdateSuccess(name));

        if let Some(action) = &data.action {
            if action.eq("save") {
                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((
                        LOCATION,
                        HeaderValue::from_str(&url)
                            .map_err(|_| error::ErrorInternalServerError(""))?,
                    ))
                    .finish());
            } else if action.eq("save_and_close") {
                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((LOCATION, HeaderValue::from_static("/files")))
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
        "is_public": { "label": is_public_str, "value": user_file.is_public, "errors": errors.is_public },
    });

    let ctx = json!({
        "ctx": layout_ctx,
        "heading": &heading,
        "file": &file,
        "user_file": &user_file,
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
            {"href": "/files", "label": translator_service.translate(lang, "page.files.index.header")},
            {"label": &heading},
        ],
        "form": {
            "action": &url,
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
    let s = template_service.render_throw_http("pages/user-files/update.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

pub fn get_edit_url(id: &str) -> String {
    let mut str_ = "/user-files/".to_string();
    str_.push_str(id);
    str_
}

impl PostData {
    pub fn prepare(&mut self) {
        prepare_value!(self._token);
        prepare_value!(self.action);
        prepare_value!(self.is_public);
    }
}

impl ErrorMessages {
    pub fn is_empty(&self) -> bool {
        self.form.is_empty() && self.is_public.is_empty()
    }
}
