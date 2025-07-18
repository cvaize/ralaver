use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::{
    prepare_upload_text_value, Alert, AlertVariant, AppService, FilePolicy, FileService,
    FileServiceError, RateLimitService, Role, RoleService, Session, TemplateService,
    TranslatableError, TranslatorService, User, UserFile, WebAuthService, WebHttpResponse,
};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::http::header::HeaderValue;
use actix_web::{
    error,
    http::{header::LOCATION, Method},
    web::{Data, ReqData},
    Error, HttpRequest, HttpResponse, Result,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use actix_multipart::Multipart;
use futures_util::{StreamExt, TryStreamExt};

#[derive(Debug, MultipartForm)]
pub struct UploadData {
    #[multipart(limit = "100MB")]
    file: TempFile,
    _token: Option<Text<String>>,
    action: Option<Text<String>>,
    is_public: Option<Text<String>>,
}

const RL_MAX_ATTEMPTS: u64 = 10;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "files_upload";

#[derive(Deserialize, Default, Debug)]
struct ErrorMessages {
    pub form: Vec<String>,
    pub file: Vec<String>,
    pub is_public: Vec<String>,
}

pub async fn show(
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
    let user_roles: Vec<Role> = role_service.all_throw_http()?;
    if !FilePolicy::can_create(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    invoke(
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

pub async fn upload(
    req: HttpRequest,
    MultipartForm(data): MultipartForm<UploadData>,
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
    let user_roles = role_service.all_throw_http()?;
    if !FilePolicy::can_create(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }
    invoke(
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

pub fn invoke(
    mut upload_form: Option<UploadData>,
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

    //
    let translator_service = translator_service.get_ref();
    let template_service = template_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let role_service = role_service.get_ref();
    let file_service = file_service.get_ref();

    let mut action_value: Option<String> = Some("save".to_string());
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
    let is_public_str = translator_service.translate(lang, "page.files.create.fields.is_public");

    let title = translator_service.translate(lang, "page.files.create.title");
    let heading = translator_service.translate(lang, "page.files.create.header");

    context_data.title = title;

    //
    let is_post = req.method().eq(&Method::POST);
    let mut user_file: Option<UserFile> = None;
    let mut is_done = false;
    let mut errors = ErrorMessages::default();

    if is_post && upload_form.is_some() {
        let mut csrf_token = None;

        if let Some(upload_form) = &upload_form {
            if let Some(Text(value)) = &upload_form._token {
                csrf_token = Some(value.to_owned());
            }
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
                if let Some(Text(value)) = &form.is_public {
                    is_public_value = value.eq("1");
                }
            }

            if errors.is_empty() {
                if let Some(form) = &upload_form {
                    let path = form
                        .file
                        .file
                        .path()
                        .to_str()
                        .ok_or(error::ErrorInternalServerError(""))?;

                    let result = file_service.upload_local_file_to_local_disk(
                        user.id,
                        path,
                        is_public_value,
                        form.file.file_name.to_owned(),
                        form.file.content_type.to_owned(),
                    );

                    if let Ok(user_file_) = result {
                        user_file = Some(user_file_);
                    } else if let Err(error) = result {
                        if error.ne(&FileServiceError::DuplicateFile) {
                            errors.form.push(error.translate(lang, translator_service));
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
        if let Some(user_file) = user_file {
            if let Some(upload_filename) = user_file.upload_filename {
                alert_variants.push(AlertVariant::FilesCreateSuccess(upload_filename));
            }
        }

        if let Some(action) = action_value {
            if action.eq("save") {
                let url_ = get_upload_url();
                return Ok(HttpResponse::SeeOther()
                    .set_alerts(alert_variants)
                    .insert_header((
                        LOCATION,
                        HeaderValue::from_str(&url_)
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
        "file": { "label": file_str, "errors": errors.file },
        "is_public": { "label": is_public_str, "value": is_public_value, "errors": errors.is_public },
    });

    let ctx = json!({
        "ctx": layout_ctx,
        "heading": &heading,
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
            {"href": "/files", "label": translator_service.translate(lang, "page.files.index.header")},
            {"label": &heading},
        ],
        "form": {
            "action": get_upload_url(),
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
    let s = template_service.render_throw_http("pages/files/upload.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

// pub async fn upload(
//     mut payload: Multipart,
//     req: HttpRequest,
//     user: ReqData<Arc<User>>,
//     session: ReqData<Arc<Session>>,
// ) -> Result<HttpResponse, Error> {
//     // iterate over multipart stream
//     while let Ok(Some(mut field)) = payload.try_next().await {
//         // dbg!(&field);
//         // let content_type = field.content_type();
//         // dbg!(content_type);
//         let content_disposition = field.content_disposition().unwrap();
//         // dbg!(content_disposition);
//         let field_name = content_disposition.get_name().unwrap().to_string();
//         // dbg!(&field_name);
//         // let filename = content_disposition.get_filename().unwrap();
//         // dbg!(filename);
//         // let filepath = format!(".{}", file_path);
//         //
//         // // File::create is blocking operation, use threadpool
//         // let mut f = web::block(|| std::fs::File::create(filepath))
//         //     .await
//         //     .unwrap();
//         //
//         // let mut bytes = BytesMut::new();
//         // // Field in turn is stream of *Bytes* object
//         while let Some(chunk) = field.next().await {
//             // let data = chunk.unwrap();
//             // bytes.extend_from_slice(&data);
//             dbg!(&field_name);
//             // dbg!(&data);
//             // filesystem operations are blocking, we have to use threadpool
//             // f = web::block(move || f.write_all(&data).map(|_| f))
//             //     .await
//             //     .unwrap();
//         }
//         // if field_name.eq("_token") {
//         //     dbg!(&bytes);
//         // }
//     }
//
//     Ok(HttpResponse::Ok()
//         .content_type(mime::APPLICATION_JSON.as_ref())
//         .body("{\"test\": 1}"))
// }

pub async fn avatar(
    mut payload: Multipart,
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        // dbg!(&field);
        // let content_type = field.content_type();
        // dbg!(content_type);
        let content_disposition = field.content_disposition().unwrap();
        // dbg!(content_disposition);
        let field_name = content_disposition.get_name().unwrap().to_string();
        // dbg!(&field_name);
        // let filename = content_disposition.get_filename().unwrap();
        // dbg!(filename);
        // let filepath = format!(".{}", file_path);
        //
        // // File::create is blocking operation, use threadpool
        // let mut f = web::block(|| std::fs::File::create(filepath))
        //     .await
        //     .unwrap();
        //
        // let mut bytes = BytesMut::new();
        // // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            // let data = chunk.unwrap();
            // bytes.extend_from_slice(&data);
            dbg!(&field_name);
            // dbg!(&data);
            // filesystem operations are blocking, we have to use threadpool
            // f = web::block(move || f.write_all(&data).map(|_| f))
            //     .await
            //     .unwrap();
        }
        // if field_name.eq("_token") {
        //     dbg!(&bytes);
        // }
    }

    Ok(HttpResponse::Ok()
        .content_type(mime::APPLICATION_JSON.as_ref())
        .body("{\"test\": 1}"))
}

pub fn get_upload_url() -> String {
    "/files/upload".to_string()
}
pub fn get_upload_avatar_url() -> String {
    "/files/upload/avatar".to_string()
}

impl UploadData {
    pub fn prepare(&mut self) {
        prepare_upload_text_value!(self._token);
        prepare_upload_text_value!(self.action);
        prepare_upload_text_value!(self.is_public);
    }
}

impl ErrorMessages {
    pub fn is_empty(&self) -> bool {
        self.form.is_empty() && self.file.is_empty() && self.is_public.is_empty()
    }
}
