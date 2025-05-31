use crate::app::controllers::web::{get_public_context_data, get_public_template_context};
use crate::app::middlewares::web_auth::REDIRECT_TO;
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{prepare_value, AlertVariant, RateLimitService, TranslatableError, WebHttpResponse};
use crate::{
    AppService, AuthService, AuthServiceError, Credentials, TemplateService, TranslatorService,
};
use actix_web::web::{Data, Form};
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;

const RL_MAX_ATTEMPTS: u64 = 5;
const RL_TTL: u64 = 60;
const RL_KEY: &'static str = "register";

#[derive(Deserialize, Debug)]
pub struct RegisterData {
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

pub async fn show(
    req: HttpRequest,
    auth_service: Data<AuthService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    rate_limit_service: Data<RateLimitService>,
) -> Result<HttpResponse, Error> {
    invoke(
        req,
        Form(RegisterData {
            email: None,
            password: None,
            confirm_password: None,
        }),
        tmpl_service,
        app_service,
        translator_service,
        auth_service,
        rate_limit_service,
    )
    .await
}

pub async fn invoke(
    req: HttpRequest,
    mut data: Form<RegisterData>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    auth_service: Data<AuthService>,
    rate_limit_service: Data<RateLimitService>,
) -> Result<HttpResponse, Error> {
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let auth_service = auth_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();

    let mut context_data = get_public_context_data(&req, translator_service, app_service);
    let lang = &context_data.lang;
    context_data.title = translator_service.translate(lang, "page.register.title");

    let email_str = translator_service.translate(lang, "page.register.fields.email");
    let password_str = translator_service.translate(lang, "page.register.fields.password");
    let confirm_password_str =
        translator_service.translate(lang, "page.register.fields.confirm_password");

    let is_post = req.method().eq(&Method::POST);
    let (is_done, form_errors, email_errors, password_errors, confirm_password_errors) = post(
        is_post,
        &req,
        &mut data,
        &email_str,
        &password_str,
        &confirm_password_str,
        translator_service,
        lang,
        auth_service,
        rate_limit_service,
    )
    .await?;

    if is_done {
        return Ok(HttpResponse::SeeOther()
            .set_alerts(vec![AlertVariant::RegisterSuccess])
            .insert_header((
                http::header::LOCATION,
                http::HeaderValue::from_static(REDIRECT_TO),
            ))
            .finish());
    }

    let layout_ctx = get_public_template_context(&context_data);
    let ctx = json!({
        "ctx": layout_ctx,
        "heading": translator_service.translate(lang, "page.register.header"),
        "form": {
            "action": "/register",
            "method": "post",
            "fields": [
                {
                    "label": email_str,
                    "type": "email",
                    "name": "email",
                    "value": &data.email,
                    "errors": email_errors,
                },
                {
                    "label": password_str,
                    "type": "password",
                    "name": "password",
                    "value": &data.password,
                    "errors": password_errors,
                },
                {
                    "label": confirm_password_str,
                    "type": "password",
                    "name": "confirm_password",
                    "value": &data.confirm_password,
                    "errors": confirm_password_errors,
                }
            ],
            "submit": {
                "label": translator_service.translate(lang, "page.register.submit"),
            },
            "reset_password": {
                "label": translator_service.translate(lang, "page.register.reset_password"),
                "href": "/reset-password",
            },
            "login": {
                "label": translator_service.translate(lang, "page.register.login"),
                "href": "/login",
            },
            "errors": form_errors,
        },
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .clear_alerts()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

async fn post(
    is_post: bool,
    req: &HttpRequest,
    data: &mut Form<RegisterData>,
    email_str: &String,
    password_str: &String,
    confirm_password_str: &String,
    translator_service: &TranslatorService,
    lang: &str,
    auth_service: &AuthService,
    rate_limit_service: &RateLimitService,
) -> Result<(bool, Vec<String>, Vec<String>, Vec<String>, Vec<String>), Error> {
    let mut is_done = false;
    let mut form_errors: Vec<String> = Vec::new();
    let mut email_errors: Vec<String> = Vec::new();
    let mut password_errors: Vec<String> = Vec::new();
    let mut confirm_password_errors: Vec<String> = Vec::new();

    if is_post {
        let rate_limit_key = rate_limit_service
            .make_key_from_request(req, RL_KEY)
            .map_err(|_| error::ErrorInternalServerError(""))?;

        let executed = rate_limit_service
            .attempt(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)
            .map_err(|_| error::ErrorInternalServerError(""))?;

        if executed {
            email_errors = Required::validated(translator_service, lang, &data.email, |value| {
                Email::validate(translator_service, lang, value, email_str)
            });
            password_errors =
                Required::validated(translator_service, lang, &data.password, |value| {
                    MinMaxLengthString::validate(
                        translator_service,
                        lang,
                        value,
                        4,
                        255,
                        password_str,
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

            is_done = if email_errors.len() == 0
                && password_errors.len() == 0
                && confirm_password_errors.len() == 0
            {
                let credentials = Credentials {
                    email: data.email.to_owned().unwrap(),
                    password: data.password.to_owned().unwrap(),
                };

                let register_result = auth_service.register_by_credentials(&credentials);

                if let Err(error) = register_result {
                    match error {
                        AuthServiceError::DuplicateEmail => {
                            email_errors.push(error.translate(lang, translator_service));
                        }
                        _ => {}
                    }
                    false
                } else {
                    true
                }
            } else {
                false
            };

            if let Some(email) = &data.email {
                if email.len() > 400 {
                    data.email = None;
                }
            }

            if let Some(password) = &data.password {
                if password.len() > 400 {
                    data.password = None;
                }
            }

            if let Some(confirm_password) = &data.confirm_password {
                if confirm_password.len() > 400 {
                    data.confirm_password = None;
                }
            }
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
    ))
}

impl RegisterData {
    pub fn prepare(&mut self) {
        prepare_value!(self.email);
        prepare_value!(self.password);
        prepare_value!(self.confirm_password);
    }
}
