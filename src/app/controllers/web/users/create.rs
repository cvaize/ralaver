use crate::app::controllers::web::{get_context_data, get_template_context};
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::{MaxLengthString, MinMaxLengthString};
use crate::app::validator::rules::required::Required;
use crate::{
    Alert, AlertVariant, AppService, Locale, LocaleService, NewUser, RateLimitService, Session,
    TemplateService, TranslatableError, TranslatorService, User, UserService, UserServiceError,
    WebAuthService, WebHttpResponse,
};
use actix_web::web::{Data, Form, ReqData};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

static RATE_LIMIT_MAX_ATTEMPTS: u64 = 10;
static RATE_LIMIT_TTL: u64 = 60;
static RATE_KEY: &str = "users_create";

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

pub async fn show(
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

pub async fn invoke(
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
    let translator_service = translator_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let user_service = user_service.get_ref();
    let locale_service = locale_service.get_ref();
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

    let is_post = req.method().eq(&Method::POST);
    let (
        is_done,
        form_errors,
        email_errors,
        password_errors,
        confirm_password_errors,
        surname_errors,
        name_errors,
        patronymic_errors,
        locale_errors,
    ) = post(
        is_post,
        &req,
        &session,
        &data,
        lang,
        &email_str,
        &password_str,
        &confirm_password_str,
        &surname_str,
        &name_str,
        &patronymic_str,
        &locale_str,
        translator_service,
        rate_limit_service,
        user_service,
        web_auth_service,
    )
    .await?;

    for form_error in form_errors {
        context_data.alerts.push(Alert::error(form_error));
    }

    if is_done {
        if let Some(email_) = &data.email {
            let user_ = user_service.first_by_email(email_);
            if let Ok(user_) = user_ {

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
                            .insert_header((
                                http::header::LOCATION,
                                header_value_back,
                            ))
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
            } else {
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

    let ctx = json!({
        "ctx": layout_ctx,
        "heading": translator_service.translate(lang, "page.users.create.header"),
        "tabs": {
            "main": translator_service.translate(lang, "page.users.create.tabs.main"),
            "extended": translator_service.translate(lang, "page.users.create.tabs.extended"),
        },
        "breadcrumbs": [
            {"href": "/", "label": translator_service.translate(lang, "page.users.create.breadcrumbs.home")},
            {"href": "/users", "label": translator_service.translate(lang, "page.users.create.breadcrumbs.users")},
            {"label": translator_service.translate(lang, "page.users.create.breadcrumbs.create")},
        ],
        "form": {
            "action": "/users/create",
            "method": "post",
            "fields": {
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
            },
            "save": translator_service.translate(lang, "page.users.create.save"),
            "save_and_close": translator_service.translate(lang, "page.users.create.save_and_close"),
            "close": {
                "label": translator_service.translate(lang, "page.users.create.close"),
                "href": "/users"
            },
        },
    });
    let s = tmpl_service.render_throw_http("pages/users/create.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

async fn post(
    is_post: bool,
    req: &HttpRequest,
    session: &Session,
    data: &Form<PostData>,
    lang: &str,
    email_str: &str,
    password_str: &str,
    confirm_password_str: &str,
    surname_str: &str,
    name_str: &str,
    patronymic_str: &str,
    locale_str: &str,
    translator_service: &TranslatorService,
    rate_limit_service: &RateLimitService,
    user_service: &UserService,
    web_auth_service: &WebAuthService,
) -> Result<
    (
        bool,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
    ),
    Error,
> {
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
        web_auth_service.check_csrf_throw_http(session, &data._token)?;

        let rate_limit_key = rate_limit_service
            .make_key_from_request(req, RATE_KEY)
            .map_err(|_| error::ErrorInternalServerError(""))?;

        let executed = rate_limit_service
            .attempt(&rate_limit_key, RATE_LIMIT_MAX_ATTEMPTS, RATE_LIMIT_TTL)
            .map_err(|_| error::ErrorInternalServerError(""))?;

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
                let new_user = NewUser {
                    email: data.email.clone().unwrap(),
                    password: data.password.clone(),
                    locale: data.locale.clone(),
                    surname: data.surname.clone(),
                    name: data.name.clone(),
                    patronymic: data.patronymic.clone(),
                };
                let result = user_service.insert(new_user);

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
            let ttl_message = rate_limit_service
                .ttl_message(translator_service, lang, &rate_limit_key)
                .map_err(|_| error::ErrorInternalServerError(""))?;
            form_errors.push(ttl_message)
        }

        if is_done {
            rate_limit_service
                .clear(&rate_limit_key)
                .map_err(|_| error::ErrorInternalServerError(""))?;
        }
    }

    Ok((
        is_done,
        form_errors,
        email_errors,
        password_errors,
        confirm_password_errors,
        surname_errors,
        name_errors,
        patronymic_errors,
        locale_errors,
    ))
}
