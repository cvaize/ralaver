use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::{MaxLengthString, MinMaxLengthString};
use crate::app::validator::rules::required::Required;
use crate::helpers::none_if_empty;
use crate::{
    Alert, AppService, Locale, LocaleService, RateLimitService, Session, TemplateService,
    TranslatableError, TranslatorService, User, UserService, UserServiceError, WebAuthService,
    WebHttpResponse,
};
use actix_web::web::Path;
use actix_web::web::{Data, Form, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

static RATE_LIMIT_MAX_ATTEMPTS: u64 = 10;
static RATE_LIMIT_TTL: u64 = 60;
static RATE_KEY: &str = "users_create_edit";

#[derive(Deserialize, Debug)]
pub struct PostData {
    pub _token: Option<String>,
    pub action: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
    pub locale: Option<String>,
    pub surname: Option<String>,
    pub name: Option<String>,
    pub patronymic: Option<String>,
}

pub async fn create(
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    invoke(
        None,
        req,
        Form(PostData {
            _token: None,
            action: None,
            email: None,
            password: None,
            confirm_password: None,
            locale: None,
            surname: None,
            name: None,
            patronymic: None,
        }),
        user,
        session,
        translator_service,
        tmpl_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
    )
    .await
}

pub async fn store(
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    invoke(
        None,
        req,
        data,
        user,
        session,
        translator_service,
        tmpl_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
    )
    .await
}

pub async fn edit(
    path: Path<u64>,
    req: HttpRequest,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    let (user_id) = path.into_inner();
    let edit_user = user_service.get_ref().first_by_id_throw_http(user_id)?;
    let post_data = PostData {
        _token: None,
        action: None,
        email: Some(edit_user.email.to_owned()),
        password: None,
        confirm_password: None,
        locale: edit_user.locale.to_owned(),
        surname: edit_user.surname.to_owned(),
        name: edit_user.name.to_owned(),
        patronymic: edit_user.patronymic.to_owned(),
    };
    invoke(
        Some(edit_user),
        req,
        Form(post_data),
        user,
        session,
        translator_service,
        tmpl_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
    )
    .await
}

pub async fn update(
    path: Path<u64>,
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    let (user_id) = path.into_inner();
    let edit_user = user_service.get_ref().first_by_id_throw_http(user_id)?;
    invoke(
        Some(edit_user),
        req,
        data,
        user,
        session,
        translator_service,
        tmpl_service,
        app_service,
        web_auth_service,
        rate_limit_service,
        user_service,
        locale_service,
    )
    .await
}

pub async fn invoke(
    edit_user: Option<User>,
    req: HttpRequest,
    data: Form<PostData>,
    user: ReqData<Arc<User>>,
    session: ReqData<Arc<Session>>,
    translator_service: Data<TranslatorService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    web_auth_service: Data<WebAuthService>,
    rate_limit_service: Data<RateLimitService>,
    user_service: Data<UserService>,
    locale_service: Data<LocaleService>,
) -> Result<HttpResponse, Error> {
    //
    let translator_service = translator_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let user_service = user_service.get_ref();
    let locale_service = locale_service.get_ref();

    //
    let user = user.as_ref();

    let mut context_data = get_context_data(
        &req,
        user,
        &session,
        translator_service,
        app_service,
        web_auth_service,
    );

    let lang = &context_data.lang;

    let email_str = translator_service.translate(lang, "page.users.create.fields.email");
    let password_str = translator_service.translate(lang, "page.users.create.fields.password");
    let confirm_password_str =
        translator_service.translate(lang, "page.users.create.fields.confirm_password");
    let surname_str = translator_service.translate(lang, "page.users.create.fields.surname");
    let name_str = translator_service.translate(lang, "page.users.create.fields.name");
    let patronymic_str = translator_service.translate(lang, "page.users.create.fields.patronymic");
    let locale_str = translator_service.translate(lang, "page.users.create.fields.locale");

    context_data.title = translator_service.translate(lang, "page.users.create.title");

    //
    let is_post = req.method().eq(&Method::POST);
    let mut is_done = false;
    let mut form_errors: Vec<String> = Vec::new();
    let mut email_errors: Vec<String> = Vec::new();
    let mut password_errors: Vec<String> = Vec::new();
    let mut confirm_password_errors: Vec<String> = Vec::new();
    let mut surname_errors: Vec<String> = Vec::new();
    let mut name_errors: Vec<String> = Vec::new();
    let mut patronymic_errors: Vec<String> = Vec::new();
    let mut locale_errors: Vec<String> = Vec::new();

    if is_post {
        web_auth_service.check_csrf_throw_http(&session, &data._token)?;

        let rate_limit_key = rate_limit_service.make_key_from_request_throw_http(&req, RATE_KEY)?;

        let executed = rate_limit_service.attempt_throw_http(
            &rate_limit_key,
            RATE_LIMIT_MAX_ATTEMPTS,
            RATE_LIMIT_TTL,
        )?;

        if executed {
            email_errors = Required::validated(translator_service, lang, &data.email, |value| {
                Email::validate(translator_service, lang, value, &email_str)
            });
            password_errors =
                Required::validated(translator_service, lang, &data.password, |value| {
                    MinMaxLengthString::validate(
                        translator_service,
                        lang,
                        value,
                        4,
                        255,
                        &password_str,
                    )
                });
            confirm_password_errors =
                Required::validated(translator_service, lang, &data.confirm_password, |value| {
                    MinMaxLengthString::validate(
                        translator_service,
                        lang,
                        value,
                        4,
                        255,
                        &confirm_password_str,
                    )
                });

            if password_errors.len() == 0 && confirm_password_errors.len() == 0 {
                let mut password_errors2: Vec<String> = Confirmed::validate(
                    translator_service,
                    lang,
                    data.password.as_ref().unwrap(),
                    data.confirm_password.as_ref().unwrap(),
                    &password_str,
                );
                confirm_password_errors.append(&mut password_errors2);
            }

            if let Some(surname) = &data.surname {
                surname_errors =
                    MaxLengthString::validate(translator_service, lang, surname, 255, &surname_str);
            }
            if let Some(name) = &data.name {
                name_errors =
                    MaxLengthString::validate(translator_service, lang, name, 255, &name_str);
            }
            if let Some(patronymic) = &data.patronymic {
                patronymic_errors = MaxLengthString::validate(
                    translator_service,
                    lang,
                    patronymic,
                    255,
                    &patronymic_str,
                );
            }
            if let Some(locale) = &data.locale {
                locale_errors =
                    MaxLengthString::validate(translator_service, lang, locale, 255, &locale_str);
            }

            is_done = if email_errors.len() == 0
                && password_errors.len() == 0
                && confirm_password_errors.len() == 0
                && surname_errors.len() == 0
                && name_errors.len() == 0
                && patronymic_errors.len() == 0
                && locale_errors.len() == 0
            {
                let mut password = none_if_empty(&data.password);
                let mut is_need_hash_password = true;
                let id = if let Some(edit_user) = &edit_user {
                    if password.is_none() {
                        is_need_hash_password = false;
                        password = edit_user.password.to_owned();
                    }
                    edit_user.id
                } else {
                    0
                };
                let mut user_data = User {
                    id,
                    email: data.email.clone().unwrap(),
                    password,
                    locale: none_if_empty(&data.locale),
                    surname: none_if_empty(&data.surname),
                    name: none_if_empty(&data.name),
                    patronymic: none_if_empty(&data.patronymic),
                };
                let result = user_service.upsert(&mut user_data, is_need_hash_password);

                if let Err(error) = result {
                    match error {
                        UserServiceError::DuplicateEmail => {
                            email_errors.push(error.translate(lang, translator_service));
                        }
                        UserServiceError::PasswordHashFail => {
                            password_errors.push(error.translate(lang, translator_service));
                        }
                        _ => {
                            form_errors.push(error.translate(lang, translator_service));
                        }
                    }
                    false
                } else {
                    true
                }
            } else {
                false
            };
        } else {
            let ttl_message = rate_limit_service.ttl_message_throw_http(
                translator_service,
                lang,
                &rate_limit_key,
            )?;
            form_errors.push(ttl_message)
        }

        if is_done {
            rate_limit_service.clear_throw_http(&rate_limit_key)?;
        }
    }

    //
    for form_error in form_errors {
        context_data.alerts.push(Alert::error(form_error));
    }

    if is_done {
        if let Some(email_) = &data.email {
            let user_ = user_service.first_by_email(email_);
            let mut is_not_found = false;
            if let Ok(user_) = user_ {
                if let Some(user_) = user_ {
                    is_not_found = true;
                    // TODO: Save alerts
                    if let Some(action) = &data.action {
                        let header_value_back = http::HeaderValue::from_static("/users");
                        if action.eq("save") {
                            let mut src = "/users/".to_string();
                            src.push_str(&user_.id.to_string());
                            return Ok(HttpResponse::SeeOther()
                                .insert_header((
                                    http::header::LOCATION,
                                    http::HeaderValue::from_str(&src).unwrap_or(header_value_back),
                                ))
                                .finish());
                        } else if action.eq("save_and_close") {
                            return Ok(HttpResponse::SeeOther()
                                .insert_header((http::header::LOCATION, header_value_back))
                                .finish());
                        }
                    }

                    let mut vars: HashMap<&str, &str> = HashMap::new();
                    let name_ = user_.get_full_name_with_id_and_email();
                    vars.insert("name", &name_);
                    context_data
                        .alerts
                        .push(Alert::success(translator_service.variables(
                            lang,
                            "alert.users.create.success",
                            &vars,
                        )));
                }
            }

            if is_not_found {
                let mut vars: HashMap<&str, &str> = HashMap::new();
                vars.insert("email", email_);
                context_data
                    .alerts
                    .push(Alert::success(translator_service.variables(
                        lang,
                        "alert.users.create.success_and_not_found",
                        &vars,
                    )));
            }
        }
    }

    let default_locale = locale_service.get_default_ref();
    let mut locales_: Vec<&Locale> = vec![default_locale];

    for locale_ in context_data.locales {
        if locale_.code.ne(&default_locale.code) {
            locales_.push(locale_);
        }
    }

    let layout_ctx = get_template_context(&context_data);

    let fields = json!({
        "email": {
            "label": email_str,
            "value": &data.email,
            "errors": email_errors,
        },
        "password": {
            "label": password_str,
            "value": &data.password,
            "errors": password_errors,
        },
        "confirm_password": {
            "label": confirm_password_str,
            "value": &data.confirm_password,
            "errors": confirm_password_errors,
        },
        "surname": {
            "label": surname_str,
            "value": &data.surname,
            "errors": surname_errors,
        },
        "name": {
            "label": name_str,
            "value": &data.name,
            "errors": name_errors,
        },
        "patronymic": {
            "label": patronymic_str,
            "value": &data.patronymic,
            "errors": patronymic_errors,
        },
        "locale": {
            "label": locale_str,
            "value": &data.locale,
            "errors": locale_errors,
            "locales": locales_
        }
    });

    let ctx = if let Some(edit_user) = &edit_user {
        let mut action = "/users/".to_string();
        action.push_str(edit_user.id.to_string().as_str());

        let full_name = edit_user.get_full_name_with_id_and_email();
        let mut vars: HashMap<&str, &str> = HashMap::new();
        vars.insert("user_name", &full_name);

        let heading = translator_service.variables(lang, "page.users.edit.header", &vars);
        json!({
            "ctx": layout_ctx,
            "heading": &heading,
            "tabs": {
                "main": translator_service.translate(lang, "page.users.create.tabs.main"),
                "extended": translator_service.translate(lang, "page.users.create.tabs.extended"),
            },
            "breadcrumbs": [
                {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
                {"href": "/users", "label": translator_service.translate(lang, "page.users.index.header")},
                {"label": &heading},
            ],
            "form": {
                "action": action,
                "method": "post",
                "fields": fields,
                "save": translator_service.translate(lang, "Save"),
                "save_and_close": translator_service.translate(lang, "Save and close"),
                "close": {
                    "label": translator_service.translate(lang, "Close"),
                    "href": "/users"
                },
            },
        })
    } else {
        json!({
            "ctx": layout_ctx,
            "heading": translator_service.translate(lang, "page.users.create.header"),
            "tabs": {
                "main": translator_service.translate(lang, "page.users.create.tabs.main"),
                "extended": translator_service.translate(lang, "page.users.create.tabs.extended"),
            },
            "breadcrumbs": [
                {"href": "/", "label": translator_service.translate(lang, "page.home.header")},
                {"href": "/users", "label": translator_service.translate(lang, "page.users.index.header")},
                {"label": translator_service.translate(lang, "page.users.create.header")},
            ],
            "form": {
                "action": "/users/create",
                "method": "post",
                "fields": fields,
                "save": translator_service.translate(lang, "Save"),
                "save_and_close": translator_service.translate(lang, "Save and close"),
                "close": {
                    "label": translator_service.translate(lang, "Close"),
                    "href": "/users"
                },
            },
        })
    };
    let s = tmpl_service.render_throw_http("pages/users/create-edit.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}
