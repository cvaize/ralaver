use crate::app::controllers::web::{get_public_context_data, get_public_template_context};
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{AlertVariant, RateLimitService, Session, WebAuthService, WebHttpResponse};
use crate::{AppService, AuthService, TemplateService, TranslatorService};
use actix_web::web::Data;
use actix_web::web::Form;
use actix_web::{error, Error, HttpRequest, HttpResponse, Result};
use http::Method;
use serde_derive::Deserialize;
use serde_json::json;

static RATE_LIMIT_MAX_ATTEMPTS: u64 = 5;
static RATE_LIMIT_TTL: u64 = 60;
static RATE_KEY: &str = "login";

#[derive(Deserialize, Debug)]
pub struct LoginData {
    pub email: Option<String>,
    pub password: Option<String>,
}

pub async fn show(
    req: HttpRequest,
    auth_service: Data<AuthService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    rate_limit_service: Data<RateLimitService>,
    web_auth_service: Data<WebAuthService>,
) -> Result<HttpResponse, Error> {
    invoke(
        req,
        Form(LoginData {
            email: None,
            password: None,
        }),
        auth_service,
        tmpl_service,
        app_service,
        translator_service,
        rate_limit_service,
        web_auth_service,
    )
    .await
}

pub async fn invoke(
    req: HttpRequest,
    mut data: Form<LoginData>,
    auth_service: Data<AuthService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    rate_limit_service: Data<RateLimitService>,
    web_auth_service: Data<WebAuthService>,
) -> Result<HttpResponse, Error> {
    let auth_service = auth_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let rate_limit_service = rate_limit_service.get_ref();
    let web_auth_service = web_auth_service.get_ref();

    let auth_result = web_auth_service.login_by_req(&req);

    if let Ok((_, token)) = auth_result {
        return Ok(HttpResponse::SeeOther()
            .cookie(web_auth_service.make_cookie_throw_http(&token)?)
            .clear_alerts()
            .insert_header((http::header::LOCATION, http::HeaderValue::from_static("/")))
            .finish());
    }

    let mut context_data = get_public_context_data(&req, translator_service, app_service);
    let lang = &context_data.lang;
    context_data.title = translator_service.translate(lang, "page.login.title");

    let email_str = translator_service.translate(lang, "page.login.fields.email");
    let password_str = translator_service.translate(lang, "page.login.fields.password");

    let is_post = req.method().eq(&Method::POST);
    let (is_done, email_errors, password_errors, form_errors, session) = post(
        is_post,
        &req,
        &mut data,
        &email_str,
        &password_str,
        lang,
        translator_service,
        auth_service,
        web_auth_service,
        rate_limit_service,
    )
    .await?;

    if is_done {
        let session = session.unwrap();
        return Ok(HttpResponse::SeeOther()
            .cookie(web_auth_service.make_cookie_throw_http(&session)?)
            .set_alerts(vec![AlertVariant::LoginSuccess])
            .insert_header((http::header::LOCATION, http::HeaderValue::from_static("/")))
            .finish());
    }

    let layout_ctx = get_public_template_context(&context_data);
    let ctx = json!({
        "ctx": layout_ctx,
        "heading": translator_service.translate(lang, "page.login.header"),
        "form": {
            "action": "/login",
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
                }
            ],
            "submit": {
                "label": translator_service.translate(lang, "page.login.submit")
            },
            "reset_password": {
                "label": translator_service.translate(lang, "page.login.reset_password"),
                "href": "/reset-password"
            },
            "register": {
                "label": translator_service.translate(lang, "page.login.register"),
                "href": "/register"
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
    data: &mut Form<LoginData>,
    email_str: &String,
    password_str: &String,
    lang: &str,
    translator_service: &TranslatorService,
    auth_service: &AuthService,
    web_auth_service: &WebAuthService,
    rate_limit_service: &RateLimitService,
) -> Result<(bool, Vec<String>, Vec<String>, Vec<String>, Option<Session>), Error> {
    let mut is_done = false;
    let mut form_errors: Vec<String> = Vec::new();
    let mut email_errors: Vec<String> = Vec::new();
    let mut password_errors: Vec<String> = Vec::new();
    let mut session: Option<Session> = None;

    if is_post {
        let rate_limit_key = rate_limit_service
            .make_key_from_request(req, RATE_KEY)
            .map_err(|_| error::ErrorInternalServerError(""))?;

        let executed = rate_limit_service
            .attempt(&rate_limit_key, RATE_LIMIT_MAX_ATTEMPTS, RATE_LIMIT_TTL)
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

            if email_errors.len() == 0 && password_errors.len() == 0 {
                let email_value = data.email.as_ref().unwrap();
                let password_value = data.password.as_ref().unwrap();
                let auth_result = auth_service.login_by_password(email_value, password_value);

                if let Ok(user_id) = auth_result {
                    let session_ = web_auth_service.generate_session(user_id);
                    web_auth_service
                        .save_session(&session_)
                        .map_err(|_| error::ErrorInternalServerError(""))?;
                    session = Some(session_);
                    is_done = true;
                } else {
                    form_errors.push(translator_service.translate(lang, "alert.login.fail"));
                }
            };

            if let Some(email) = &data.email {
                if email.len() > 400 {
                    data.email = None;
                }
            }

            if password_errors.len() != 0 {
                data.password = None;
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

    Ok((is_done, email_errors, password_errors, form_errors, session))
}
