use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::app::validator::rules::length::MinMaxLengthString as MMLS;
use crate::app::validator::rules::required::Required;
use crate::libs::actix_web::types::form::Form;
use crate::{prepare_upload_text_value, prepare_value, Alert, AlertVariant, AppService, Disk, File, FileColumn, FilePolicy, FileService, FileServiceError, RateLimitService, Role, RoleService, Session, TemplateService, TranslatableError, TranslatorService, UploadData, User, UserServiceError, WebAuthService, WebHttpResponse};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::web::Path;
use actix_web::web::{Data, ReqData};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde::Deserialize;
use serde_derive::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use mime::Mime;
use strum_macros::{Display, EnumString};

// TODO: Remove temp file if error exists
#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
    _token: Option<Text<String>>,
    action: Option<Text<String>>,
    name: Option<Text<String>>,
    upload_disk: Option<Text<String>>,
    is_public: Option<Text<String>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PostData {
    _token: Option<String>,
    action: Option<String>,
    url: Option<String>,
    name: Option<String>,
    external_disk: Option<String>,
    is_public: Option<String>,
}

const RL_MAX_ATTEMPTS: u64 = 10;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "files_create";

#[derive(Deserialize, Default, Debug)]
struct ErrorMessages {
    pub form: Vec<String>,
    pub file: Vec<String>,
    pub name: Vec<String>,
    pub upload_disk: Vec<String>,
    pub external_disk: Vec<String>,
    pub url: Vec<String>,
    pub is_public: Vec<String>,
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
    invoke(
        None,
        None,
        req,
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
        Some(data),
        req,
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

pub async fn upload(
    req: HttpRequest,
    MultipartForm(form): MultipartForm<UploadForm>,
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
        Some(form),
        None,
        req,
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
    mut upload_form: Option<UploadForm>,
    mut post_form: Option<Form<PostData>>,
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
    if let Some(mut upload_form_) = upload_form {
        upload_form_.prepare();
        upload_form = Some(upload_form_);
    }
    if let Some(mut post_form_) = post_form {
        post_form_.prepare();
        post_form = Some(post_form_);
    }

    //
    let translator_service = translator_service.get_ref();
    let template_service = template_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let role_service = role_service.get_ref();
    let file_service = file_service.get_ref();

    let mut form_type: &str = "multipart_file";
    let mut action_value: Option<String> = Some("save".to_string());
    let mut name_value: Option<String> = None;
    let mut upload_disk_value: Option<String> = None;
    let mut external_disk_value: Option<String> = None;
    let mut url_value: Option<String> = None;
    let mut is_public_value: bool = false;

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

    let file_str = translator_service.translate(lang, "page.files.create.fields.file");
    let name_str = translator_service.translate(lang, "page.files.create.fields.name");
    let is_public_str = translator_service.translate(lang, "page.files.create.fields.is_public");
    let disk_str = translator_service.translate(lang, "page.files.create.fields.disk.label");
    let url_str = translator_service.translate(lang, "page.files.create.fields.url");

    let title = translator_service.translate(lang, "page.files.create.title");
    let heading = translator_service.translate(lang, "page.files.create.header");

    context_data.title = title;

    //
    let is_post = req.method().eq(&Method::POST);
    let mut is_done = false;
    let mut errors = ErrorMessages::default();

    if is_post && (upload_form.is_some() || post_form.is_some()) {
        let mut csrf_token = None;

        if let Some(upload_form) = &upload_form {
            if let Some(Text(value)) = &upload_form._token {
                csrf_token = Some(value.to_owned());
            }
        } else if let Some(post_form) = &post_form {
            csrf_token = post_form._token.to_owned();
        }

        web_auth_service.check_csrf_throw_http(&session, &csrf_token)?;

        let rate_limit_key = rate_limit_service.make_key_from_request_throw_http(&req, RL_KEY)?;

        let executed =
            rate_limit_service.attempt_throw_http(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)?;

        if executed {
            if let Some(form) = &upload_form {
                if let Some(Text(value)) = &form.action {
                    action_value = Some(value.to_owned());
                }
                if let Some(Text(value)) = &form.name {
                    name_value = Some(value.to_owned());
                }
                if let Some(Text(value)) = &form.upload_disk {
                    upload_disk_value = Some(value.to_owned());
                }
                if let Some(Text(value)) = &form.is_public {
                    is_public_value = value.eq("1");
                }
                // TODO: In list validation
                errors.upload_disk =
                    Required::validate(translator_service, lang, &upload_disk_value, &disk_str);
            } else if let Some(form) = &post_form {
                form_type = "external_file";
                action_value = form.action.to_owned();
                name_value = form.name.to_owned();
                external_disk_value = form.external_disk.to_owned();
                url_value = form.url.to_owned();
                if let Some(is_public) = &form.is_public {
                    is_public_value = is_public.eq("1");
                }
                // TODO: In list validation
                errors.external_disk =
                    Required::validate(translator_service, lang, &external_disk_value, &disk_str);
                // TODO: Convert url and validate
                errors.url = Required::validated(
                    translator_service,
                    lang,
                    &url_value,
                    |value| MMLS::validate(translator_service, lang, value, 10, 2048, &url_str),
                    &url_str,
                );
            }

            if let Some(value) = &name_value {
                errors.name = MMLS::validate(translator_service, lang, value, 1, 255, &name_str);
            }

            if errors.is_empty() {
                if let Some(form) = &upload_form {
                    dbg!(form);
                    let path = form.file.file.path().to_str().ok_or(error::ErrorInternalServerError(""))?;

                    let to_disk = upload_disk_value.to_owned().ok_or(error::ErrorInternalServerError(""))?;
                    let to_disk = Disk::from_str(&to_disk).map_err(|_| error::ErrorInternalServerError(""))?;

                    let filename = if name_value.is_some() {
                        name_value.to_owned()
                    } else {
                        form.file.file_name.to_owned()
                    };
                    let upload_data = UploadData {
                        mime: form.file.content_type.to_owned(),
                        filename,
                        size: None,
                        hash: None,
                        is_public: Some(is_public_value),
                        creator_user_id: Some(user.id),
                    };


                    if to_disk.eq(&Disk::Local) {
                        let result = file_service.upload_local_file_to_local_disk(path, upload_data);

                        if let Err(error) = result {
                            if error.ne(&FileServiceError::DuplicateLocalPath) {
                                errors.form.push(error.translate(lang, translator_service));
                            }
                        }
                    }

                } else if let Some(form) = &post_form {
                    dbg!(form);

                    let url = url_value.to_owned().ok_or(error::ErrorInternalServerError(""))?;

                    let to_disk = external_disk_value.to_owned().ok_or(error::ErrorInternalServerError(""))?;
                    let to_disk = Disk::from_str(&to_disk).map_err(|_| error::ErrorInternalServerError(""))?;

                    let upload_data = UploadData {
                        mime: None,
                        filename: None,
                        size: None,
                        hash: None,
                        is_public: Some(is_public_value),
                        creator_user_id: Some(user.id),
                    };

                    if to_disk.eq(&Disk::External) {
                        let result = file_service.upload_external_file_to_external_disk(&url, upload_data);

                        if let Err(error) = result {
                            if error.ne(&FileServiceError::DuplicateLocalPath) {
                                errors.form.push(error.translate(lang, translator_service));
                            }
                        }
                    }
                }
                is_done = errors.is_empty();
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
        if let Some(action) = action_value {
            if action.eq("save") {
                let mut url_ = get_upload_url();
                if form_type.eq("external_file") {
                    url_ = get_create_from_external_url();
                }
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

    if upload_disk_value.is_none() {
        upload_disk_value = Some(Disk::Local.to_string());
    }

    if external_disk_value.is_none() {
        external_disk_value = Some(Disk::External.to_string());
    }

    let upload_disk_options = json!([
        {
            "label": translator_service.translate(lang, "page.files.create.fields.disk.options.local"),
            "value": Disk::Local.to_string(),
        }
    ]);

    let external_disk_options = json!([
        {
            "label": translator_service.translate(lang, "page.files.create.fields.disk.options.external"),
            "value": Disk::External.to_string(),
        }
    ]);

    for variant in &alert_variants {
        context_data
            .alerts
            .push(Alert::from_variant(translator_service, lang, variant));
    }

    let layout_ctx = get_template_context(&context_data);

    let fields = json!({
        "file": { "label": file_str, "errors": errors.file },
        "name": { "label": name_str, "value": name_value, "errors": errors.name },
        "upload_disk": { "label": disk_str, "value": upload_disk_value, "errors": errors.upload_disk, "options": upload_disk_options },
        "external_disk": { "label": disk_str, "value": external_disk_value, "errors": errors.external_disk, "options": external_disk_options },
        "url": { "label": url_str, "value": url_value, "errors": errors.url },
        "is_public": { "label": is_public_str, "value": is_public_value, "errors": errors.is_public },
    });

    let ctx = json!({
        "ctx": layout_ctx,
        "heading": &heading,
        "tabs": {
            "upload": translator_service.translate(lang, "page.files.create.tabs.upload"),
            "external": translator_service.translate(lang, "page.files.create.tabs.external"),
        },
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
            {"href": "/files", "label": translator_service.translate(lang, "page.files.index.header")},
            {"label": &heading},
        ],
        "form": {
            "upload_action": get_upload_url(),
            "external_action": get_create_from_external_url(),
            "type": form_type,
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
    let s = template_service.render_throw_http("pages/files/create.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

pub fn get_upload_url() -> String {
    "/files/upload".to_string()
}

pub fn get_create_from_external_url() -> String {
    "/files/create-from-external-url".to_string()
}

impl PostData {
    pub fn prepare(&mut self) {
        prepare_value!(self._token);
        prepare_value!(self.action);
        prepare_value!(self.url);
        prepare_value!(self.name);
        prepare_value!(self.external_disk);
        prepare_value!(self.is_public);
    }
}

impl UploadForm {
    pub fn prepare(&mut self) {
        prepare_upload_text_value!(self._token);
        prepare_upload_text_value!(self.action);
        prepare_upload_text_value!(self.name);
        prepare_upload_text_value!(self.upload_disk);
        prepare_upload_text_value!(self.is_public);
    }
}

impl ErrorMessages {
    pub fn is_empty(&self) -> bool {
        self.form.is_empty()
    }
}
