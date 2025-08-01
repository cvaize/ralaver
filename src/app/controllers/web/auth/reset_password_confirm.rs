use crate::app::controllers::web::auth::reset_password::CODE_LEN;
use crate::app::controllers::web::{get_public_context_data, get_public_template_context};
use crate::app::middlewares::web_auth::REDIRECT_TO;
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::str_min_max_chars_count::StrMinMaxCharsCount;
use crate::app::validator::rules::required::Required;
use crate::{prepare_value, AlertVariant, RateLimitService, UserService, WebHttpResponse, RESET_PASSWORD_TTL};
use crate::{AppService, AuthService, TemplateService, TranslatorService};
use actix_web::web::{Data, Form, Query};
use actix_web::{
    error,
    http::{header::LOCATION, Method},
    Error, HttpRequest, HttpResponse, Result,
};
use actix_web::http::header::HeaderValue;
use serde_derive::Deserialize;
use serde_json::json;

const RL_MAX_ATTEMPTS: u64 = 5;
const RL_TTL: u64 = RESET_PASSWORD_TTL;
const RL_KEY: &'static str = "reset_password_confirm";

#[derive(Deserialize)]
pub struct ResetPasswordConfirmQuery {
    pub email: Option<String>,
    pub code: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ResetPasswordConfirmData {
    pub code: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

pub async fn show(
    req: HttpRequest,
    query: Query<ResetPasswordConfirmQuery>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    auth_service: Data<AuthService>,
    user_service: Data<UserService>,
    rate_limit_service: Data<RateLimitService>,
) -> Result<HttpResponse, Error> {
    invoke(
        req,
        query,
        Form(ResetPasswordConfirmData {
            code: None,
            email: None,
            password: None,
            confirm_password: None,
        }),
        tmpl_service,
        app_service,
        translator_service,
        auth_service,
        user_service,
        rate_limit_service,
    )
    .await
}

pub async fn invoke(
    req: HttpRequest,
    query: Query<ResetPasswordConfirmQuery>,
    mut data: Form<ResetPasswordConfirmData>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    auth_service: Data<AuthService>,
    user_service: Data<UserService>,
    rate_limit_service: Data<RateLimitService>,
) -> Result<HttpResponse, Error> {
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let auth_service = auth_service.get_ref();
    let user_service = user_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();

    let query = query.into_inner();

    let mut context_data = get_public_context_data(&req, translator_service, app_service);
    let lang = &context_data.lang;
    context_data.title = translator_service.translate(lang, "page.reset_password_confirm.title");

    let email_str = translator_service.translate(lang, "page.reset_password_confirm.fields.email");
    let password_str =
        translator_service.translate(lang, "page.reset_password_confirm.fields.password");
    let confirm_password_str =
        translator_service.translate(lang, "page.reset_password_confirm.fields.confirm_password");

    if let Some(email) = query.email {
        data.email = Some(email.to_owned());
    }
    if let Some(code) = query.code {
        data.code = Some(code.to_owned());
    }

    let mut email = data.email.to_owned().unwrap_or("".to_string());
    let mut code = data.code.to_owned().unwrap_or("".to_string());

    if email.len() > 400 {
        email = "".to_string();
    }
    if code.len() != CODE_LEN {
        code = "".to_string();
    }

    let action = format!("/reset-password-confirm?code={}&email={}", code, email);

    let is_post = req.method().eq(&Method::POST);
    let (is_done, form_errors, email_errors, password_errors, confirm_password_errors, code_errors) =
        post(
            is_post,
            &req,
            &mut data,
            &email_str,
            &password_str,
            &confirm_password_str,
            translator_service,
            lang,
            auth_service,
            user_service,
            rate_limit_service,
        )
        .await?;

    if is_done {
        return Ok(HttpResponse::SeeOther()
            .set_alerts(vec![AlertVariant::ResetPasswordConfirmSuccess])
            .insert_header((LOCATION, HeaderValue::from_static(REDIRECT_TO)))
            .finish());
    }

    if code_errors.len() != 0 || data.email.is_none() || data.code.is_none() {
        return Ok(HttpResponse::SeeOther()
            .set_alerts(vec![AlertVariant::ResetPasswordConfirmCodeNotEqual])
            .insert_header((LOCATION, HeaderValue::from_static("/reset-password")))
            .finish());
    }

    let layout_ctx = get_public_template_context(&context_data);
    let ctx = json!({
        "ctx": layout_ctx,
        "heading": translator_service.translate(lang, "page.reset_password_confirm.header"),
        "back": {
            "label": translator_service.translate(lang, "page.reset_password_confirm.back"),
            "href": "/reset-password",
        },
        "form": {
            "action": action,
            "method": "post",
            "header": translator_service.translate(lang, "page.reset_password_confirm.header"),
            "fields": [
                {
                    "name": "code",
                    "type": "hidden",
                    "value": code,
                },
                {
                    "label": email_str,
                    "type": "email",
                    "name": "email",
                    "readonly": "readonly",
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
                "label": translator_service.translate(lang, "page.reset_password_confirm.submit"),
            },
            "errors": form_errors
        },
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok()
        .content_type(mime::TEXT_HTML_UTF_8.as_ref())
        .body(s))
}

async fn post(
    is_post: bool,
    req: &HttpRequest,
    data: &mut Form<ResetPasswordConfirmData>,
    email_str: &String,
    password_str: &String,
    confirm_password_str: &String,
    translator_service: &TranslatorService,
    lang: &str,
    auth_service: &AuthService,
    user_service: &UserService,
    rate_limit_service: &RateLimitService,
) -> Result<
    (
        bool,
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
    let mut code_errors: Vec<String> = Vec::new();

    if is_post {
        let rate_limit_key = rate_limit_service
            .make_key_from_request(req, RL_KEY)
            .map_err(|_| error::ErrorInternalServerError(""))?;

        let executed = rate_limit_service
            .attempt(&rate_limit_key, RL_MAX_ATTEMPTS, RL_TTL)
            .map_err(|_| error::ErrorInternalServerError(""))?;

        if executed {
            email_errors = Required::validated(
                translator_service,
                lang,
                &data.email,
                |value| Email::validate(translator_service, lang, value, &email_str),
                &email_str,
            );
            password_errors = Required::validated(
                translator_service,
                lang,
                &data.password,
                |value| {
                    StrMinMaxCharsCount::validate(
                        translator_service,
                        lang,
                        value,
                        4,
                        255,
                        &password_str,
                    )
                },
                &password_str,
            );
            confirm_password_errors = Required::validated(
                translator_service,
                lang,
                &data.confirm_password,
                |value| {
                    StrMinMaxCharsCount::validate(
                        translator_service,
                        lang,
                        value,
                        4,
                        255,
                        &confirm_password_str,
                    )
                },
                &confirm_password_str,
            );
            code_errors = Required::validated(
                translator_service,
                lang,
                &data.code,
                |value| {
                    StrMinMaxCharsCount::validate(
                        translator_service,
                        lang,
                        value,
                        CODE_LEN,
                        CODE_LEN,
                        "code",
                    )
                },
                "code",
            );

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
                && code_errors.len() == 0
            {
                let mut is_done1 = false;
                let d_ = "".to_string();
                let email = data.email.as_ref().unwrap_or(&d_);
                let code = data.code.as_ref().unwrap_or(&d_);
                let password = data.password.as_ref().unwrap_or(&d_);

                let is_exists_code: bool = auth_service
                    .is_exists_reset_password_code(email, code)
                    .map_err(|_| error::ErrorInternalServerError(""))?;

                if is_exists_code {
                    user_service
                        .update_password_by_email(email, password)
                        .map_err(|_| error::ErrorInternalServerError(""))?;
                    auth_service
                        .delete_reset_password_code(email, code)
                        .map_err(|_| error::ErrorInternalServerError(""))?;

                    is_done1 = true;
                } else {
                    code_errors.push("Reset password code not exists.".to_string());
                }

                is_done1
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

            if let Some(code) = &data.code {
                if code.len() > 400 {
                    data.code = None;
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
        code_errors,
    ))
}

impl ResetPasswordConfirmData {
    pub fn prepare(&mut self) {
        prepare_value!(self.email);
        prepare_value!(self.code);
        prepare_value!(self.password);
        prepare_value!(self.confirm_password);
    }
}

impl ResetPasswordConfirmQuery {
    pub fn prepare(&mut self) {
        prepare_value!(self.email);
        prepare_value!(self.code);
    }
}
